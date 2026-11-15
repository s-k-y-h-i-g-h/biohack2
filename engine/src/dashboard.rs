//! spec-003 Phase 5 — panel data derivation (T241–T244), zero web
//! dependencies by construction (spec 005 FR-081: the CLI builds with the
//! web feature off; here there is no web feature to turn off).
//!
//! `panel_view` turns stored data into exactly what a renderer needs —
//! gauge reading, band series, target edges, now marker position — so the
//! web component and the future terminal renderer share one derivation
//! instead of each re-deriving (and drifting). Per spec 005 the code lives
//! in the engine until a second consumer proves the crate boundary; the
//! spec itself mandates extracting the shared crate only *after* the first
//! working panel, not before.

use crate::explain::limitations_text;
use crate::models::{LogEntry, ModelClass, TargetBand};
use crate::pharmacokinetics::{sample_series, ActiveDosePoint, DoseEvent};
use crate::pk_params::SubstanceModel;
use crate::redose;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Panel window (T243): 24 h back, 12 h forward, hourly.
pub const LOOKBACK_HOURS: i64 = 24;
pub const LOOKFORWARD_HOURS: i64 = 12;
pub const STEP_HOURS: f64 = 1.0;

/// One readable line on the axis at `now` — the gauge number set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NowReading {
    pub low: f64,
    pub expected: f64,
    pub high: f64,
    pub unit: String,
}

/// Everything a panel renders, derived from stored data. The renderer draws
/// this and computes nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelView {
    pub substance: String,
    pub model_class: ModelClass,
    /// Series over [now-24h, now+12h] at hourly steps (T243). Empty when
    /// the substance is not simulatable — the panel then shows `note`.
    pub series: Vec<ActiveDosePoint>,
    /// Index within `series` of the point nearest `now` (T243 marker).
    pub now_index: usize,
    /// The band at now (gauge). None ⇒ no model ⇒ render `note`, not zero.
    pub now_reading: Option<NowReading>,
    /// Target band edges to draw on the same axis (T244).
    pub target: Option<TargetBand>,
    /// True when the expected value sits below the target's low edge.
    pub below_target: bool,
    /// True when the band width is an assumed ±25% rather than a catalog
    /// range — the renderer flags it visually.
    pub band_is_assumption: bool,
    /// Why there is no curve ("no model — {reason}", T239 surface), or the
    /// limitations text when there is one.
    pub note: String,
}

/// Derive the panel view for one substance as of `now`.
///
/// Non-simulatable models are NOT an error: an empty series plus the
/// recorded reason is the required rendering (A1/AC-3), and a renderer
/// that special-cased Err would be tempted to draw zero.
pub fn panel_view(
    model: &SubstanceModel,
    entries: &[LogEntry],
    target: Option<&TargetBand>,
    now: DateTime<Utc>,
) -> Result<PanelView, String> {
    let key = model.substance.trim().to_lowercase();
    let doses: Vec<DoseEvent> = entries
        .iter()
        .filter(|e| e.name.trim().to_lowercase() == key)
        .filter_map(|e| DoseEvent::from_log_entry(e).ok())
        .collect();

    let simulatable = match model.model_class {
        ModelClass::None => false,
        ModelClass::Plateau => true,
        _ => model.has_numeric_half_life(),
    };

    if !simulatable {
        return Ok(PanelView {
            substance: model.substance.clone(),
            model_class: model.model_class,
            series: Vec::new(),
            now_index: 0,
            now_reading: None,
            target: target.cloned(),
            below_target: false,
            band_is_assumption: false,
            note: format!("No model — {}", limitations_text(model)),
        });
    }

    let start = now - chrono::Duration::hours(LOOKBACK_HOURS);
    let end = now + chrono::Duration::hours(LOOKFORWARD_HOURS);
    let series = sample_series(model, &doses, start, end, STEP_HOURS)?;
    let band_is_assumption = series.first().is_some_and(|p| p.band_is_assumption);
    // Hourly steps from now-24h: the point nearest `now` is index 24 by
    // construction; clamp so a future window change cannot panic renderers.
    let now_index = (LOOKBACK_HOURS as usize).min(series.len().saturating_sub(1));
    let now_reading = series.get(now_index).map(|p| NowReading {
        low: p.low,
        expected: p.expected,
        high: p.high,
        unit: p.unit.clone(),
    });
    let below_target = match (&target, &now_reading) {
        (Some(b), Some(_)) => {
            // Same future limb the redose alert judges — share the predicate.
            redose::is_below_target_now(&series[now_index..], b.low)
        }
        _ => false,
    };

    Ok(PanelView {
        substance: model.substance.clone(),
        model_class: model.model_class,
        series,
        now_index,
        now_reading,
        target: target.cloned(),
        below_target,
        band_is_assumption,
        note: limitations_text(model),
    })
}

/// T250 — verify the shared panel-derivation fixture (`tests/fixtures/
/// panel_vectors.json`) through the real dashboard path, `panel_view`.
/// Same contract as `pharmacokinetics::check_vectors`: an empty return
/// means this target reproduces every recorded value within tolerance.
/// Consumed by the native x86_64 test AND the wasm32 node runner, making
/// panel data derivation (window, indices, band edges, empty-model path)
/// a bit-exact-within-tolerance parity claim — SC-033, not just the raw
/// curve maths.
pub fn check_panel_vectors(json: &str) -> Vec<String> {
    use crate::models::{ItemType, TargetBand};
    use chrono::TimeZone;
    #[derive(serde::Deserialize)]
    struct Fix {
        tolerance: f64,
        vectors: Vec<Vector>,
    }
    #[derive(serde::Deserialize)]
    struct Vector {
        name: String,
        model: SubstanceModel,
        doses: Vec<RawDose>,
        checks: Vec<Chk>,
        #[serde(default)]
        band: Option<RawBand>,
        #[serde(default)]
        expect_empty: bool,
        #[serde(default)]
        band_is_assumption: Option<bool>,
        #[serde(default)]
        note_prefix: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct RawDose {
        amount: f64,
        unit: String,
        offset_hours: f64,
    }
    #[derive(serde::Deserialize)]
    struct RawBand {
        substance: String,
        low: f64,
        high: f64,
        window_hours: f64,
    }
    #[derive(serde::Deserialize)]
    struct Chk {
        index: usize,
        low: f64,
        expected: f64,
        high: f64,
    }

    let fix: Fix = match serde_json::from_str(json) {
        Ok(f) => f,
        Err(e) => return vec![format!("fixture parse error: {e}")],
    };
    // Fixed instants so native and wasm see identical arithmetic: doses
    // are offset from `now`; the window is the panel's own now-24..now+12.
    let epoch = match Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return vec!["fixture epoch is not a single UTC instant".into()],
    };
    let now = epoch + chrono::Duration::hours(48);
    let mut failures = Vec::new();
    for v in &fix.vectors {
        let entries: Vec<crate::models::LogEntry> = v
            .doses
            .iter()
            .map(|d| crate::models::LogEntry {
                id: uuid::Uuid::nil(),
                user_id: "fixture".into(),
                item_type: ItemType::Supplement,
                item_id: None,
                name: v.model.substance.clone(),
                quantity: Some(d.amount),
                unit: Some(d.unit.clone()),
                route: None,
                timestamp: now
                    + chrono::Duration::milliseconds((d.offset_hours * 3_600_000.0).round() as i64),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            })
            .collect();
        let band = v.band.as_ref().map(|b| TargetBand {
            substance: b.substance.clone(),
            low: b.low,
            high: b.high,
            window_hours: b.window_hours,
            updated_at: now,
        });
        let view = match panel_view(&v.model, &entries, band.as_ref(), now) {
            Ok(view) => view,
            Err(e) => {
                failures.push(format!("{}: panel_view failed: {e}", v.name));
                continue;
            }
        };
        if v.expect_empty {
            if !view.series.is_empty() {
                failures.push(format!("{}: expected empty series, got {}", v.name, view.series.len()));
            }
            if let Some(prefix) = &v.note_prefix {
                if !view.note.starts_with(prefix.as_str()) {
                    failures.push(format!("{}: note {:?} lacks prefix {prefix:?}", v.name, view.note));
                }
            }
            continue;
        }
        let want_len = (LOOKBACK_HOURS + LOOKFORWARD_HOURS) as usize + 1;
        if view.series.len() != want_len {
            failures.push(format!("{}: series len {}, want {want_len}", v.name, view.series.len()));
            continue;
        }
        if view.now_index != LOOKBACK_HOURS as usize {
            failures.push(format!("{}: now_index {}, want {LOOKBACK_HOURS}", v.name, view.now_index));
        }
        if let Some(flag) = v.band_is_assumption {
            if view.series[0].band_is_assumption != flag {
                failures.push(format!(
                    "{}: band_is_assumption {}, want {flag}",
                    v.name, view.series[0].band_is_assumption
                ));
            }
        }
        for c in &v.checks {
            let Some(pt) = view.series.get(c.index) else {
                failures.push(format!("{}: no sample at index {}", v.name, c.index));
                continue;
            };
            for (label, got, want) in [
                ("low", pt.low, c.low),
                ("expected", pt.expected, c.expected),
                ("high", pt.high, c.high),
            ] {
                if (got - want).abs() > fix.tolerance {
                    failures.push(format!(
                        "{}: idx {} {label}: got {got}, want {want} (tol {})",
                        v.name, c.index, fix.tolerance
                    ));
                }
            }
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemType;
    use uuid::Uuid;

    fn model() -> SubstanceModel {
        SubstanceModel {
            substance: "Caffeine".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(5.0),
            half_life_range: Some((4.0, 6.0)),
            params: serde_json::Value::Null,
            provenance: "test".into(),
            catalog_version: 1,
            rationale: None,
        }
    }

    fn dose(age_hours: i64, qty: f64, now: DateTime<Utc>) -> LogEntry {
        LogEntry {
            id: Uuid::new_v4(),
            user_id: "local-device".into(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: "caffeine".into(),
            quantity: Some(qty),
            unit: Some("mg".into()),
            route: None,
            timestamp: now - chrono::Duration::hours(age_hours),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        }
    }

    fn band(low: f64, high: f64, now: DateTime<Utc>) -> TargetBand {
        TargetBand {
            substance: "Caffeine".into(),
            low,
            high,
            window_hours: 24.0,
            updated_at: now,
        }
    }

    #[test]
    fn window_now_marker_and_gauge_are_derived() {
        let now = DateTime::parse_from_rfc3339("2026-11-15T12:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let entries = vec![dose(2, 200.0, now), dose(8, 100.0, now)];
        let v = panel_view(&model(), &entries, Some(&band(50.0, 150.0, now)), now).unwrap();
        // 24 back + 12 forward, hourly, inclusive endpoints.
        assert_eq!(v.series.len(), (LOOKBACK_HOURS + LOOKFORWARD_HOURS + 1) as usize);
        assert_eq!(v.now_index, LOOKBACK_HOURS as usize);
        // Series must actually bracket now at the marker.
        let marker = &v.series[v.now_index];
        assert_eq!(marker.t, now);
        // 300 mg on board at now ⇒ above a 50–150 target, not below.
        let r = v.now_reading.clone().expect("gauge reading");
        assert!(r.expected > 50.0, "{r:?}");
        assert!(!v.below_target);
        assert_eq!(r.unit, "mg");
        assert!(!v.band_is_assumption); // catalog range present
    }

    #[test]
    fn stale_doses_put_the_reading_below_target() {
        let now = DateTime::parse_from_rfc3339("2026-11-15T12:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        // One small dose long gone: expected ≈ 0 at now.
        let entries = vec![dose(30, 50.0, now)];
        let v = panel_view(&model(), &entries, Some(&band(50.0, 150.0, now)), now).unwrap();
        assert!(v.below_target, "{:?}", v.now_reading);
    }

    #[test]
    fn unmodelled_renders_note_not_error_not_zero() {
        let now = Utc::now();
        let m = SubstanceModel::unmodelled("Weird Stuff", "no published PK", 3);
        let v = panel_view(&m, &[], None, now).expect("not an error");
        assert!(v.series.is_empty());
        assert!(v.now_reading.is_none(), "no model must not render a zero level");
        assert!(v.note.starts_with("No model — "), "{}", v.note);
        assert!(v.note.contains("no published PK"));
    }

    #[test]
    fn other_substances_do_not_leak_into_the_view() {
        let now = DateTime::parse_from_rfc3339("2026-11-15T12:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let mut mg = dose(1, 400.0, now);
        mg.name = "Magnesium".into();
        let v = panel_view(&model(), &[mg], None, now).unwrap();
        // Same window, but every point is zero: no caffeine events.
        assert_eq!(v.series.len(), (LOOKBACK_HOURS + LOOKFORWARD_HOURS + 1) as usize);
        let r = v.now_reading.expect("gauge exists even at zero");
        assert_eq!(r.expected, 0.0);
    }

    #[test]
    fn panel_vectors_fixture_passes_and_pins_closed_forms() {
        // The shared T250 fixture (also run wasm-side by the node runner).
        let fixture = include_str!("../tests/fixtures/panel_vectors.json");
        let failures = crate::dashboard::check_panel_vectors(fixture);
        assert!(
            failures.is_empty(),
            "panel vector failures:\n{}",
            failures.join("\n")
        );
        use chrono::TimeZone;
        // Independent analytic spot-checks: the fixture must not merely
        // record whatever the engine emitted — these are hand-computed.
        // decay t_half=6h, dose 100 at now-6h: exactly one half-life to now.
        let m = SubstanceModel {
            substance: "VecPanelDecay".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(6.0),
            half_life_range: None,
            params: serde_json::Value::Null,
            provenance: "test".into(),
            catalog_version: 1,
            rationale: None,
        };
        let now = chrono::Utc
            .with_ymd_and_hms(2026, 1, 3, 0, 0, 0)
            .unwrap();
        let e = LogEntry {
            id: uuid::Uuid::nil(),
            user_id: "test".into(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: "VecPanelDecay".into(),
            quantity: Some(100.0),
            unit: Some("mg".into()),
            route: None,
            timestamp: now - chrono::Duration::hours(6),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        let v = crate::dashboard::panel_view(&m, &[e], None, now).unwrap();
        assert_eq!(v.series[24].expected, 50.0, "0.5^1 is exact");
        assert_eq!(v.series[18].expected, 100.0);
    }
}

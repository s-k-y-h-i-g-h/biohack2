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
}

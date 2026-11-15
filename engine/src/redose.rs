//! spec-003 Phase 3 — pure, deterministic detection layer.
//!
//! Two jobs, both side-effect free so they run identically native and in
//! wasm (same discipline as `pharmacokinetics`):
//!
//! * T227 — target-band crossing detection: the simulated band falling
//!   below a configured `TargetBand::low` yields the crossing time and the
//!   redose window (FR-034, AC-1).
//! * T230 — rolling-window cumulative dose per substance, the input for
//!   the catalog daily-max check behind the `CumulativeLoad` alert (FR-035).
//!
//! Alert *construction* and persistence deliberately live on the existing
//! alert path (T228 added the types; T229/T231 wire generation into the
//! save flow) — nothing here forks it.

use crate::pharmacokinetics::{ActiveDosePoint, DoseEvent};
use chrono::{DateTime, Duration, Utc};

/// FR-034: the simulated band crossed below the target's lower edge.
///
/// The redose window is the honest uncertainty range for *when* the true
/// level fell below target, read off the band edges:
/// * `redose_window_start` — earliest moment the true level could be below
///   target (the `low` edge crosses; fastest-decay scenario).
/// * `redose_window_end` — moment the true level is *certainly* below
///   target (the `high` edge crosses; slowest-decay scenario).
///
/// Redosing inside the window is the FR-034 recommendation: before the end
/// the level may already be sub-therapeutic; after the end it certainly is.
#[derive(Debug, Clone, PartialEq)]
pub struct BandCrossing {
    pub substance: String,
    pub target_low: f64,
    /// When the `expected` curve passes below `target_low`.
    pub crossed_at: DateTime<Utc>,
    pub redose_window_start: DateTime<Utc>,
    pub redose_window_end: DateTime<Utc>,
}

/// First downward crossing of `threshold` by the given curve, with the
/// exact instant linearly interpolated between the bracketing samples.
/// A series that starts below the threshold counts as already crossed.
fn first_crossing(
    series: &[ActiveDosePoint],
    threshold: f64,
    value: fn(&ActiveDosePoint) -> f64,
) -> Option<DateTime<Utc>> {
    let first = series.first()?;
    if value(first) < threshold {
        return Some(first.t);
    }
    for pair in series.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let (va, vb) = (value(a), value(b));
        if va >= threshold && vb < threshold {
            // va >= threshold > vb, so va - vb > 0 and the ratio is in [0, 1).
            let frac = (va - threshold) / (va - vb);
            let span_ms = (b.t - a.t).num_milliseconds() as f64;
            return Some(a.t + Duration::milliseconds((frac * span_ms).round() as i64));
        }
    }
    None
}

/// T227: detect the first target-band crossing in a sampled series.
///
/// Returns `None` when the band never sinks below the target (which is the
/// "no alert" case of AC-3's sibling: no crossing, no redose alert). A
/// restored level (fresh dose logged, series re-sampled) simply stops
/// producing a crossing — the basis for resolution in T232.
pub fn detect_band_crossing(
    series: &[ActiveDosePoint],
    substance: &str,
    target_low: f64,
) -> Option<BandCrossing> {
    if !target_low.is_finite() || series.is_empty() {
        return None;
    }
    let crossed_at = first_crossing(series, target_low, |p| p.expected)?;
    let last_t = series.last().map(|p| p.t)?;
    let redose_window_start = first_crossing(series, target_low, |p| p.low).unwrap_or(crossed_at);
    // If even the slowest-decay edge is still in range by the series end,
    // certainty hasn't been reached inside the horizon: the window runs to
    // the last sampled instant.
    let redose_window_end = first_crossing(series, target_low, |p| p.high).unwrap_or(last_t);
    // Band edges satisfy low <= expected <= high by construction; clamp to
    // guard the invariant rather than trust it across float rounding.
    Some(BandCrossing {
        substance: substance.to_string(),
        target_low,
        crossed_at,
        redose_window_start: redose_window_start.min(crossed_at),
        redose_window_end: redose_window_end.max(crossed_at),
    })
}

/// T232 predicate: is the substance currently below the target's lower
/// edge? Alert resolution is about the level *now* (a logged dose that
/// restores the band resolves the alert); `detect_band_crossing` remains
/// the forward projection used to raise it.
pub fn is_below_target_now(series: &[ActiveDosePoint], target_low: f64) -> bool {
    series.first().is_some_and(|p| p.expected < target_low)
}

/// T230 / FR-035: total of `substance`'s doses in the half-open window
/// `(at - window_hours, at]` — out-of-window and future doses contribute
/// nothing. Contributions are summed in deterministic order (timestamp,
/// then entry id), so native and wasm see bit-identical float accumulation
/// regardless of how the caller ordered the log.
///
/// Amounts are summed in the log's stored unit; the caller is responsible
/// for comparing against the catalog max **in the same unit**.
pub fn rolling_dose(
    doses: &[DoseEvent],
    substance: &str,
    at: DateTime<Utc>,
    window_hours: f64,
) -> Result<f64, String> {
    if !window_hours.is_finite() || window_hours <= 0.0 {
        return Err(format!(
            "rolling window must be finite and > 0 h, got {window_hours}"
        ));
    }
    let ms = (window_hours * 3_600_000.0).round() as i64;
    let start = at - Duration::milliseconds(ms);
    let mut selected: Vec<&DoseEvent> = doses
        .iter()
        .filter(|d| d.substance == substance && d.timestamp > start && d.timestamp <= at)
        .collect();
    selected.sort_by(|a, b| {
        a.timestamp
            .cmp(&b.timestamp)
            .then_with(|| a.entry_id.cmp(&b.entry_id))
    });
    Ok(selected.iter().map(|d| d.amount).sum())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelClass;
    use crate::pharmacokinetics::sample_series;
    use crate::pk_params::SubstanceModel;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap()
    }

    fn model(hl: f64) -> SubstanceModel {
        SubstanceModel {
            substance: "Mag".into(),
            model_class: ModelClass::Decay,
            half_life_hours: Some(hl),
            half_life_range: None,
            params: serde_json::Value::Null,
            provenance: "test".into(),
            catalog_version: 1,
            rationale: None,
        }
    }

    fn dose(amount: f64, at: DateTime<Utc>) -> DoseEvent {
        DoseEvent {
            substance: "Mag".into(),
            amount,
            unit: "mg".into(),
            timestamp: at,
            entry_id: Uuid::new_v4().to_string(),
        }
    }

    fn series_at(doses: &[DoseEvent], from: DateTime<Utc>, hours: i64) -> Vec<ActiveDosePoint> {
        sample_series(&model(6.0), doses, from, from + Duration::hours(hours), 0.1)
            .expect("sampling works")
    }

    // T234 (subset): the crossing fires at the analytic time.
    // 100 mg, t½ 6 h, target low 40 mg -> cross at 6*log2(100/40) ≈ 15.85 h.
    #[test]
    fn crossing_fires_at_analytic_time() {
        let doses = [dose(100.0, t0())];
        let crossing = detect_band_crossing(&series_at(&doses, t0(), 48), "Mag", 40.0)
            .expect("band sinks below 40 mg");
        let exact =
            t0() + Duration::milliseconds((6.0f64 * (100.0f64 / 40.0).log2() * 3.6e6) as i64);
        let drift = (crossing.crossed_at - exact).num_seconds().abs();
        assert!(drift <= 120, "crossing off by {drift}s from analytic");
    }

    // T234 (subset): a target the curve never reaches produces no alert.
    #[test]
    fn no_crossing_when_target_low_is_below_the_curve() {
        let doses = [dose(100.0, t0())];
        // 36 h of decay bottoms out at 1.56 mg — above a 1.0 mg target.
        assert!(detect_band_crossing(&series_at(&doses, t0(), 36), "Mag", 1.0).is_none());
        assert!(detect_band_crossing(&[], "Mag", 40.0).is_none());
        assert!(detect_band_crossing(&series_at(&doses, t0(), 48), "Mag", f64::NAN).is_none());
    }

    // T234 (subset) / T232 basis: logging a dose that restores the level
    // makes the forward series stop crossing — resolution without state.
    #[test]
    fn restore_resolves_the_crossing() {
        let dose1 = dose(100.0, t0());
        let now = t0() + Duration::hours(20); // comfortably below 40 mg
        let before = series_at(std::slice::from_ref(&dose1), now, 24);
        assert!(detect_band_crossing(&before, "Mag", 40.0).is_some());

        assert!(is_below_target_now(&before, 40.0));

        let redose = dose(150.0, now);
        let doses = [dose1, redose];
        let after = series_at(&doses, now, 24);
        assert!(
            !is_below_target_now(&after, 40.0),
            "a restoring dose must bring the level back into band"
        );
        // (The projection will legitimately find the *next* future drop —
        // that is a fresh crossing, not the unresolved alert.)
    }

    // T234 (subset): window edges — exactly-at-start excluded (half-open),
    // future excluded, order of the log irrelevant.
    #[test]
    fn rolling_window_edges_and_ordering() {
        let at = t0();
        let in_a = dose(10.0, at - Duration::hours(1));
        let edge = dose(20.0, at - Duration::hours(24)); // == start -> excluded
        let just_in = dose(30.0, at - Duration::hours(24) + Duration::seconds(1));
        let future = dose(40.0, at + Duration::seconds(1));
        let other = {
            let mut d = dose(50.0, at);
            d.substance = "Zma".into();
            d
        };
        let at_itself = dose(60.0, at); // boundary: included
        let doses = [in_a, edge, just_in, future, other, at_itself];
        // In window (t0-24h, t0]: in_a (10) + just_in (30) + at_itself (60).
        assert_eq!(rolling_dose(&doses, "Mag", at, 24.0).unwrap(), 100.0);
        // Determinism: reversed input, identical result.
        let rev: Vec<_> = doses.iter().rev().cloned().collect();
        assert_eq!(rolling_dose(&rev, "Mag", at, 24.0).unwrap(), 100.0);
        // Shift `at` forward 1 h: edge/just_in drop out, `future` comes in.
        assert_eq!(
            rolling_dose(&doses, "Mag", at + Duration::hours(1), 24.0).unwrap(),
            110.0
        );
        assert!(rolling_dose(&doses, "Mag", at, 0.0).is_err());
    }

    // Band-invariant ordering inside the result.
    #[test]
    fn redose_window_brackets_the_expected_crossing() {
        let doses = [dose(100.0, t0())];
        let c = detect_band_crossing(&series_at(&doses, t0(), 72), "Mag", 40.0).unwrap();
        assert!(c.redose_window_start <= c.crossed_at);
        assert!(c.crossed_at <= c.redose_window_end);
    }
}

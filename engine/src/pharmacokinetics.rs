//! Half-life string parsing (spec 003 Phase 1, T203).
//!
//! The seed catalog stores half-life as free text (`"24h"`, `"5-6 hours"`,
//! `"~8d"`, `"unknown"`). Everything downstream — model classes, active-dose
//! bands, redose windows — needs a NUMBER in hours. This module is the only
//! place free text becomes a number, and it fails loudly: an unparseable
//! string yields an explicit failure reason, never a silent default.
//!
//! Range policy: a range like `"5-6 hours"` keeps BOTH endpoints. It is not
//! collapsed to a midpoint — band computation happens downstream from the
//! endpoints (FR-033), so collapsing here would destroy the uncertainty the
//! source text carries.

use crate::models::CatalogItem;

/// Outcome of parsing one `half_life` string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParsedHalfLife {
    /// Single value, hours.
    Hours(f64),
    /// Range, hours (low, high) — endpoints preserved.
    Range(f64, f64),
}

/// Parse one catalog `half_life` string into hours.
///
/// Accepted shapes (case-insensitive, surrounding/certainty tildes ignored):
/// - `"24h"`, `"4 hr"`, `"6 hours"` — single value in hours
/// - `"~8d"`, `"1.5 days"` — single value in days (× 24)
/// - `"5-6 hours"`, `"~4-6h (zero-order ~7 g/h)"` — range; a trailing
///   parenthetical annotation is ignored for parsing purposes
/// - `"3 weeks"` — single value in weeks (× 168)
///
/// Anything else (`"unknown"`, `"N/A"`, `"varies"`, malformed numbers) is
/// `Err(reason)` with the raw string quoted — the caller collects these into
/// the explicit failure list (T204).
pub fn parse_half_life(raw: &str) -> Result<ParsedHalfLife, String> {
    let fail = |why: &str| Err(format!("'{raw}': {why}"));

    // Drop a trailing parenthetical annotation, e.g. "(zero-order ~7 g/h)".
    let base = match raw.find('(') {
        Some(i) => raw[..i].trim_end(),
        None => raw,
    };
    // Certainty markers are noise for the parser; the raw string keeps them
    // in the failure report.
    let base = base.trim().trim_start_matches('~').trim();
    if base.is_empty() {
        return fail("empty after stripping certainty marker/annotation");
    }

    // Split off the unit token: expect "<number>[ - <number>] <unit>" or
    // "<number><unit>" (e.g. "24h", "~8d").
    let (nums_part, unit) =
        split_number_unit(base).ok_or_else(|| format!("'{raw}': no recognisable number+unit"))?;

    let factor = match unit.to_ascii_lowercase().as_str() {
        "h" | "hr" | "hrs" | "hour" | "hours" => 1.0,
        "d" | "day" | "days" => 24.0,
        "w" | "wk" | "wks" | "week" | "weeks" => 168.0,
        other => return fail(&format!("unknown time unit '{other}'")),
    };

    // Range or single? Allow "5-6", "5 - 6".
    if let Some((lo_s, hi_s)) = nums_part.split_once('-') {
        let lo = parse_num(lo_s).ok_or_else(|| format!("'{raw}': bad range low '{lo_s}'"))?;
        let hi = parse_num(hi_s).ok_or_else(|| format!("'{raw}': bad range high '{hi_s}'"))?;
        if lo <= 0.0 || hi <= 0.0 {
            return fail(&format!("'{raw}': non-positive range endpoints"));
        }
        if lo > hi {
            return fail(&format!("'{raw}': reversed range {lo_s}-{hi_s}"));
        }
        return Ok(ParsedHalfLife::Range(lo * factor, hi * factor));
    }

    let v = parse_num(&nums_part).ok_or_else(|| format!("'{raw}': bad number '{nums_part}'"))?;
    if v <= 0.0 {
        return fail("non-positive half-life");
    }
    Ok(ParsedHalfLife::Hours(v * factor))
}

/// Extract (numeric part, unit part) from a cleaned half-life core like
/// `"24h"`, `"5-6 hours"`, `"1.5days"`. Numeric part is digits, dots, and an
/// optional range dash; unit is the remainder.
fn split_number_unit(s: &str) -> Option<(String, String)> {
    let bytes = s.as_bytes();
    let mut end = 0usize; // exclusive end of numeric part
    while end < bytes.len() {
        let c = bytes[end] as char;
        if c.is_ascii_digit() || c == '.' || c == '-' || c == ' ' {
            end += 1;
        } else {
            break;
        }
    }
    let nums = s[..end].trim();
    let unit = s[end..].trim();
    if nums.is_empty() || unit.is_empty() {
        return None;
    }
    let compact = nums.replace(' ', "");
    // At most one dash (a range), and interior spaces are only legal when a
    // dash joins the two sides: "5 - 6" is fine, "5 6" is not.
    if compact.matches('-').count() > 1 {
        return None;
    }
    if nums.contains(' ') && !compact.contains('-') {
        return None;
    }
    Some((compact, unit.to_string()))
}

/// Parse a plain decimal number; rejects anything else.
fn parse_num(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    // f64::from_str accepts "inf"/"NaN" — forbid them explicitly.
    if !t.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Derive the numeric `CatalogItem` half-life fields from the raw string.
/// Used by BOTH construction paths (seed `make_item` and DB row mapping) so
/// round-tripped items compare equal — one derivation, no drift.
/// Failures produce `(None, None)`: absence is honest; the failure list comes
/// from `audit_catalog`, never from a silent zero.
pub fn half_life_fields(raw: Option<&str>) -> (Option<f64>, Option<(f64, f64)>) {
    match raw.map(parse_half_life) {
        Some(Ok(ParsedHalfLife::Hours(h))) => (Some(h), None),
        Some(Ok(ParsedHalfLife::Range(lo, hi))) => (None, Some((lo, hi))),
        _ => (None, None),
    }
}

/// Audit result for the whole seed catalog (T204).
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogPkAudit {
    /// (name, hours) for items parsed to a single value.
    pub single: Vec<(String, f64)>,
    /// (name, low, high) for items parsed to a range.
    pub ranges: Vec<(String, f64, f64)>,
    /// (name, reason) for items that FAILED to parse — explicit, never hidden.
    pub failures: Vec<(String, String)>,
}

/// Run the parser over every seed-catalog item, bucketing results.
/// Every failure here becomes a `ModelClass::None` assignment with the
/// recorded reason (T204/T205).
pub fn audit_catalog(catalog: &[CatalogItem]) -> CatalogPkAudit {
    let mut audit = CatalogPkAudit {
        single: vec![],
        ranges: vec![],
        failures: vec![],
    };
    for item in catalog {
        let raw = item.half_life.as_deref().unwrap_or("");
        match parse_half_life(raw) {
            Ok(ParsedHalfLife::Hours(h)) => audit.single.push((item.name.clone(), h)),
            Ok(ParsedHalfLife::Range(lo, hi)) => audit.ranges.push((item.name.clone(), lo, hi)),
            Err(reason) => audit.failures.push((item.name.clone(), reason)),
        }
    }
    audit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_hour_forms() {
        assert_eq!(parse_half_life("24h"), Ok(ParsedHalfLife::Hours(24.0)));
        assert_eq!(parse_half_life("4 hr"), Ok(ParsedHalfLife::Hours(4.0)));
        assert_eq!(parse_half_life("6 hours"), Ok(ParsedHalfLife::Hours(6.0)));
    }

    #[test]
    fn parses_days_and_weeks_into_hours() {
        assert_eq!(parse_half_life("~8d"), Ok(ParsedHalfLife::Hours(192.0)));
        assert_eq!(parse_half_life("1.5 days"), Ok(ParsedHalfLife::Hours(36.0)));
        assert_eq!(parse_half_life("3 weeks"), Ok(ParsedHalfLife::Hours(504.0)));
    }

    #[test]
    fn parses_ranges_preserving_endpoints() {
        assert_eq!(
            parse_half_life("5-6 hours"),
            Ok(ParsedHalfLife::Range(5.0, 6.0))
        );
        assert_eq!(
            parse_half_life("5 - 6 h"),
            Ok(ParsedHalfLife::Range(5.0, 6.0))
        );
        // The alcohol form from the real catalog: range + annotation.
        assert_eq!(
            parse_half_life("~4-6h (zero-order ~7 g/h)"),
            Ok(ParsedHalfLife::Range(4.0, 6.0))
        );
    }

    #[test]
    fn rejects_unknown_forms_with_explicit_reasons() {
        for bad in [
            "unknown",
            "N/A",
            "varies",
            "",
            "~",
            "abc",
            "5 monkeys",
            "0h",
            "-3h",
            "5-6",
            "h5",
        ] {
            let r = parse_half_life(bad);
            assert!(r.is_err(), "expected failure for '{bad}', got {r:?}");
            // Every failure reason must quote the raw string.
            assert!(r.unwrap_err().contains(bad) || bad.is_empty());
        }
    }

    #[test]
    fn rejects_reversed_ranges() {
        assert!(parse_half_life("6-5 hours").is_err());
    }

    #[test]
    fn catalog_audit_enumerates_every_item() {
        let catalog = crate::catalog::seed_catalog();
        let audit = audit_catalog(&catalog);
        let total = audit.single.len() + audit.ranges.len() + audit.failures.len();
        assert_eq!(
            total,
            catalog.len(),
            "every item must land in exactly one bucket"
        );

        // The one known failure in today's seed catalog: 9-Me-BC ("unknown").
        // If this set changes, the change must be deliberate — update with a
        // reason, never silently.
        let failure_names: Vec<&str> = audit.failures.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            failure_names,
            vec!["9-Me-BC"],
            "unexpected half-life parse failures: {failure_names:?}"
        );

        // Spot-check the conversions that matter clinically.
        let caffeine = audit
            .single
            .iter()
            .find(|(n, _)| n == "Caffeine")
            .unwrap()
            .1;
        assert_eq!(caffeine, 5.0);
        let estradiol = audit
            .single
            .iter()
            .find(|(n, _)| n == "Estradiol Enanthate")
            .unwrap()
            .1;
        assert_eq!(estradiol, 192.0); // ~8d
        let alcohol = audit.ranges.iter().find(|(n, ..)| n == "Alcohol").unwrap();
        assert_eq!((alcohol.1, alcohol.2), (4.0, 6.0));
    }
}

// ── Simulation (spec 003 Phase 2, T210–T218) ────────────────────────────────
//
// Deterministic, offline, closed-form (FR-032). Same f64 maths on every
// target: no ODE solver, no randomness, no wall-clock reads inside the
// curves. Band always (FR-033): no code path here produces a lone line.

use crate::models::LogEntry;
use crate::pk_params::SubstanceModel;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Documented default variability for SINGLE-VALUE half-lives (T218, FR-033
/// / A5): the band is evaluated at t½ × (1 ± 0.25) and the output is marked
/// assumption-derived. Rationale recorded in
/// `specs/003-substance-core-pharmacokinetics/research.md`.
pub const DEFAULT_VARIABILITY: f64 = 0.25;

/// Upper bound on sampled points per series — a guard against accidental
/// multi-year windows at minute resolution (per-substance render windowing
/// is T219; this is only the memory fuse).
pub const MAX_SERIES_POINTS: usize = 10_000;

/// One dose, normalised for simulation. Derived from a `LogEntry` only —
/// the log is the single source of truth (FR-030 lineage).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoseEvent {
    pub substance: String,
    pub amount: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    /// Source log entry id, so any curve point is traceable to its rows.
    pub entry_id: String,
}

impl DoseEvent {
    /// Derive from a log entry. Entries without a usable quantity or unit
    /// are REJECTED, not defaulted — a dose event with a guessed amount or a
    /// guessed "mg" is exactly the silent default this spec forbids.
    pub fn from_log_entry(entry: &LogEntry) -> Result<Self, String> {
        let amount = entry.quantity.ok_or_else(|| {
            format!(
                "{}: log entry has no quantity — cannot simulate a dose from it",
                entry.name
            )
        })?;
        if !amount.is_finite() || amount <= 0.0 {
            return Err(format!(
                "{}: quantity {amount} is not a positive finite amount",
                entry.name
            ));
        }
        let unit = entry
            .unit
            .clone()
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty())
            .ok_or_else(|| format!("{}: log entry has no unit", entry.name))?;
        Ok(DoseEvent {
            substance: entry.name.clone(),
            amount,
            unit,
            timestamp: entry.timestamp,
            entry_id: entry.id.to_string(),
        })
    }
}

/// One rendered sample of the on-board curve (spec.md Data Model:
/// `ActiveDosePoint { t, low, expected, high, unit }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveDosePoint {
    pub t: DateTime<Utc>,
    pub low: f64,
    pub expected: f64,
    pub high: f64,
    pub unit: String,
    /// True when the band came from DEFAULT_VARIABILITY rather than a
    /// catalog range — the panel MUST state this (FR-033 / A5 / AC-2).
    pub band_is_assumption: bool,
}

/// Case-folded unit key for comparison only. There is deliberately NO unit
/// conversion anywhere: a mismatch blocks the series (T211).
fn unit_key(unit: &str) -> String {
    unit.trim().to_ascii_lowercase()
}

/// First-order one-compartment decay, superposed over all doses at or before
/// `t` (T212 + T216): amount × 0.5^(Δt / t½), summed. Doses after `t`
/// contribute nothing (causality).
pub fn onboard_at(doses: &[DoseEvent], half_life_hours: f64, t: DateTime<Utc>) -> f64 {
    debug_assert!(half_life_hours > 0.0);
    let mut total = 0.0;
    for d in doses {
        if d.timestamp > t {
            continue;
        }
        let dt_hours = (t - d.timestamp).num_milliseconds() as f64 / 3_600_000.0;
        total += d.amount * 0.5f64.powf(dt_hours / half_life_hours);
    }
    total
}

/// Resolve the three half-lives the band is evaluated at from a model row:
/// (low_curve_hl, expected_hl, high_curve_hl, band_is_assumption).
/// A catalog RANGE is used directly (fast half-life bounds the `low` curve,
/// slow bounds `high`). A single value is widened by DEFAULT_VARIABILITY and
/// flagged as an assumption.
fn band_half_lives(model: &SubstanceModel) -> Result<(f64, f64, f64, bool), String> {
    if let Some((lo, hi)) = model.half_life_range {
        if lo <= 0.0 || hi < lo {
            return Err(format!(
                "{}: invalid half-life range {lo}..{hi}",
                model.substance
            ));
        }
        let expected = match model.half_life_hours {
            Some(h) if h > 0.0 => h,
            // Geometric mean keeps the expected value multiplicative-symmetric
            // between the endpoints; arithmetic mean would bias long t½.
            // Also the fallback for a null/invalid nominal outside the range.
            _ => (lo * hi).sqrt(),
        };
        return Ok((lo, expected, hi, false));
    }
    let nominal = model.half_life_hours.filter(|h| *h > 0.0).ok_or_else(|| {
        format!(
            "{}: modelled as {:?} but has no usable half-life",
            model.substance, model.model_class
        )
    })?;
    Ok((
        nominal * (1.0 - DEFAULT_VARIABILITY),
        nominal,
        nominal * (1.0 + DEFAULT_VARIABILITY),
        true,
    ))
}

/// Public sampling API for `decay` substances (T220 shape; classes beyond
/// decay land in T213–T215 and return an explicit not-implemented error
/// here until then — never a silent decay fallback).
///
/// Units (T211): every dose must share one case-folded unit, and if the
/// model's `params` carry an `expected_unit` string it must match too.
/// Mismatches block the whole series with the mismatch in the message; no
/// silent conversion happens anywhere.
pub fn sample_series(
    model: &SubstanceModel,
    doses: &[DoseEvent],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    step_hours: f64,
) -> Result<Vec<ActiveDosePoint>, String> {
    if model.model_class != crate::models::ModelClass::Decay {
        return Err(format!(
            "{}: model class {:?} simulation not implemented yet (T213–T215)",
            model.substance, model.model_class
        ));
    }
    if end < start {
        return Err("series window end precedes start".into());
    }
    if !step_hours.is_finite() || step_hours <= 0.0 {
        return Err(format!(
            "step_hours {step_hours} is not a positive finite number"
        ));
    }
    let (hl_low, hl_expected, hl_high, band_is_assumption) = band_half_lives(model)?;

    // ── unit validation (T211): one unit across doses, model unit if declared
    let mut dose_unit: Option<String> = None;
    for d in doses {
        match &dose_unit {
            None => dose_unit = Some(d.unit.clone()),
            Some(u) if unit_key(u) == unit_key(&d.unit) => {}
            Some(u) => {
                return Err(format!(
                    "{}: mixed dose units '{}' and '{}' (entry {}) — refusing to sum across units",
                    model.substance, u, d.unit, d.entry_id
                ));
            }
        }
    }
    if let Some(declared) = model.params.get("expected_unit").and_then(|v| v.as_str()) {
        if let Some(u) = &dose_unit {
            if unit_key(declared) != unit_key(u) {
                return Err(format!(
                    "{}: dose unit '{u}' does not match model unit '{declared}' — blocked, no conversion (T211)",
                    model.substance
                ));
            }
        }
    }
    let unit = dose_unit.unwrap_or_else(|| "dose".into());

    let step_ms = (step_hours * 3_600_000.0).round() as i64;
    let total_ms = (end - start).num_milliseconds();
    let n = (total_ms / step_ms) as usize + 1;
    if n > MAX_SERIES_POINTS {
        return Err(format!(
            "series would emit {n} points (> {MAX_SERIES_POINTS}); widen step_hours or narrow the window"
        ));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = start + Duration::milliseconds(step_ms * i as i64);
        out.push(ActiveDosePoint {
            t,
            low: onboard_at(doses, hl_low, t).min(onboard_at(doses, hl_high, t)),
            expected: onboard_at(doses, hl_expected, t),
            high: onboard_at(doses, hl_low, t).max(onboard_at(doses, hl_high, t)),
            unit: unit.clone(),
            band_is_assumption,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod sim_tests {
    use super::*;
    use crate::models::{ItemType, ModelClass, RouteType};
    use chrono::TimeZone;
    use uuid::Uuid;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 1, 12, 0, 0).unwrap()
    }

    fn entry(name: &str, qty: Option<f64>, unit: Option<&str>, at: DateTime<Utc>) -> LogEntry {
        LogEntry {
            id: Uuid::new_v4(),
            user_id: "u1".into(),
            item_type: ItemType::Supplement,
            item_id: None,
            name: name.into(),
            quantity: qty,
            unit: unit.map(|u| u.into()),
            route: Some(RouteType::Oral),
            timestamp: at,
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        }
    }

    fn dose(amount: f64, at: DateTime<Utc>) -> DoseEvent {
        DoseEvent {
            substance: "Mag".into(),
            amount,
            unit: "mg".into(),
            timestamp: at,
            entry_id: "e".into(),
        }
    }

    fn model(nominal: Option<f64>, range: Option<(f64, f64)>) -> SubstanceModel {
        SubstanceModel {
            substance: "Mag".into(),
            model_class: ModelClass::Decay,
            half_life_hours: nominal,
            half_life_range: range,
            params: serde_json::Value::Null,
            provenance: "test".into(),
            catalog_version: 1,
            rationale: None,
        }
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn half_life_is_half_the_dose() {
        let doses = vec![dose(100.0, t0())];
        let series = sample_series(
            &model(Some(6.0), None),
            &doses,
            t0(),
            t0() + Duration::hours(6),
            1.0,
        )
        .expect("series");
        assert_eq!(series.len(), 7);
        assert!(approx(series[0].expected, 100.0));
        assert!(approx(series[6].expected, 50.0)); // t = t½
        assert!(series[6].band_is_assumption); // single value => widened band
        assert!(series[6].low < 50.0 && series[6].high > 50.0);
        // ±25 % documented edges: 0.5^(6/4.5) and 0.5^(6/7.5)
        assert!(approx(series[6].low, 100.0 * 0.5f64.powf(6.0 / 4.5)));
        assert!(approx(series[6].high, 100.0 * 0.5f64.powf(6.0 / 7.5)));
    }

    #[test]
    fn two_doses_superpose() {
        let doses = vec![dose(100.0, t0()), dose(100.0, t0() + Duration::hours(6))];
        let series = sample_series(
            &model(Some(6.0), None),
            &doses,
            t0(),
            t0() + Duration::hours(6),
            6.0,
        )
        .expect("series");
        assert!(approx(series[0].expected, 100.0));
        assert!(approx(series[1].expected, 150.0)); // 50 (first) + 100 (second)
    }

    #[test]
    fn future_doses_contribute_nothing() {
        let doses = vec![dose(100.0, t0()), dose(500.0, t0() + Duration::hours(10))];
        let on = onboard_at(&doses, 6.0, t0() + Duration::hours(3));
        assert!(approx(on, 100.0 * 0.5f64.powf(0.5)));
    }

    #[test]
    fn catalog_range_band_is_used_without_assumption_flag() {
        let doses = vec![dose(100.0, t0())];
        let series = sample_series(
            &model(Some(5.0), Some((4.0, 6.0))),
            &doses,
            t0(),
            t0() + Duration::hours(5),
            5.0,
        )
        .expect("series");
        assert!(approx(series[1].expected, 50.0)); // expected uses nominal 5h
        assert!(!series[1].band_is_assumption); // range came from catalog
        assert!(approx(series[1].low, 100.0 * 0.5f64.powf(5.0 / 4.0)));
        assert!(approx(series[1].high, 100.0 * 0.5f64.powf(5.0 / 6.0)));
    }

    #[test]
    fn mixed_units_block_no_conversion() {
        let mut d2 = dose(100.0, t0());
        d2.unit = "mcg".into();
        let doses = vec![dose(100.0, t0()), d2];
        let err = sample_series(
            &model(Some(6.0), None),
            &doses,
            t0(),
            t0() + Duration::hours(1),
            1.0,
        )
        .expect_err("must block");
        assert!(err.contains("mixed dose units"), "{err}");
    }

    #[test]
    fn model_unit_mismatch_blocks() {
        let mut m = model(Some(6.0), None);
        m.params = serde_json::json!({ "expected_unit": "mg" });
        let mut d = dose(100.0, t0());
        d.unit = "mcg".into();
        let err =
            sample_series(&m, &[d], t0(), t0() + Duration::hours(1), 1.0).expect_err("must block");
        assert!(err.contains("blocked, no conversion"), "{err}");
    }

    #[test]
    fn entries_without_quantity_or_unit_are_rejected() {
        let e = entry("Mag", None, Some("mg"), t0());
        assert!(
            DoseEvent::from_log_entry(&e)
                .unwrap_err()
                .contains("no quantity")
        );
        let e = entry("Mag", Some(100.0), None, t0());
        assert!(
            DoseEvent::from_log_entry(&e)
                .unwrap_err()
                .contains("no unit")
        );
        let e = entry("Mag", Some(-3.0), Some("mg"), t0());
        assert!(
            DoseEvent::from_log_entry(&e)
                .unwrap_err()
                .contains("positive finite")
        );
        let ok = entry("Mag", Some(100.0), Some(" mg "), t0());
        assert_eq!(DoseEvent::from_log_entry(&ok).unwrap().unit, "mg");
    }

    #[test]
    fn unmodelled_classes_refuse_instead_of_defaulting() {
        let mut m = model(Some(6.0), None);
        m.model_class = ModelClass::None;
        let err = sample_series(&m, &[dose(1.0, t0())], t0(), t0() + Duration::hours(1), 1.0)
            .expect_err("must refuse");
        assert!(err.contains("not implemented"), "{err}");
    }

    #[test]
    fn window_guard_rejects_absurd_resolution() {
        let doses = vec![dose(100.0, t0())];
        let err = sample_series(
            &model(Some(6.0), None),
            &doses,
            t0(),
            t0() + Duration::days(30),
            1.0 / 3600.0, // one-second steps over a month
        )
        .expect_err("must refuse");
        assert!(
            err.contains("MAX_SERIES_POINTS") || err.contains("points"),
            "{err}"
        );
    }

    #[test]
    fn invalid_window_and_step_are_errors() {
        let doses = vec![dose(100.0, t0())];
        let m = model(Some(6.0), None);
        assert!(sample_series(&m, &doses, t0(), t0() - Duration::hours(1), 1.0).is_err());
        assert!(sample_series(&m, &doses, t0(), t0() + Duration::hours(1), 0.0).is_err());
    }
}

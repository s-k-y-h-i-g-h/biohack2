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
use std::f64::consts::LN_2 as LN2;

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
/// Shape parameters resolved once per series from `model.params`. Shape
/// parameters that the math genuinely needs (absorption rate, rise time)
/// must exist in the catalog — they are REFUSED when absent, never invented.
/// Documented optional defaults: `lag_hours = 0` (lag is a modifier, the
/// Bateman maths is identical at zero lag).
struct SimShape {
    lag_hours: f64,
    /// `oral_absorption`: absorption-rate half-life → k_a = ln2 / this.
    absorption_half_life_hours: Option<f64>,
    /// `plateau`: time for one dose's contribution to approach its own
    /// plateau. Required for that class.
    rise_time_hours: Option<f64>,
    /// `plateau`: optional total body-pool ceiling (FR-031 saturation).
    ceiling: Option<f64>,
}

fn param_f64(params: &serde_json::Value, key: &str) -> Option<f64> {
    params.get(key).and_then(|v| v.as_f64())
}

impl SimShape {
    fn resolve(model: &SubstanceModel) -> Result<Self, String> {
        use crate::models::ModelClass;
        let p = &model.params;
        match model.model_class {
            ModelClass::Decay | ModelClass::Accumulation => Ok(SimShape {
                lag_hours: 0.0,
                absorption_half_life_hours: None,
                rise_time_hours: None,
                ceiling: param_f64(p, "ceiling").filter(|c| *c > 0.0),
            }),
            ModelClass::OralAbsorption => {
                let abs_hl = param_f64(p, "absorption_half_life_hours").or_else(|| {
                    param_f64(p, "ka_per_hour").map(|ka| if ka > 0.0 { LN2 / ka } else { -1.0 })
                });
                let abs_hl = abs_hl
                    .filter(|v| *v > 0.0 && v.is_finite())
                    .ok_or_else(|| {
                        format!(
                            "{}: oral_absorption needs an explicit absorption parameter \
                             (absorption_half_life_hours or ka_per_hour) in catalog params — \
                             refusing rather than guessing an absorption rate",
                            model.substance
                        )
                    })?;
                let lag = param_f64(p, "lag_hours").unwrap_or(0.0);
                if !lag.is_finite() || lag < 0.0 {
                    return Err(format!("{}: lag_hours {lag} is invalid", model.substance));
                }
                Ok(SimShape {
                    lag_hours: lag,
                    absorption_half_life_hours: Some(abs_hl),
                    rise_time_hours: None,
                    ceiling: None,
                })
            }
            ModelClass::Plateau => {
                let rise = param_f64(p, "rise_time_hours")
                    .filter(|v| *v > 0.0 && v.is_finite())
                    .ok_or_else(|| {
                        format!(
                            "{}: plateau needs an explicit rise_time_hours in catalog params \
                             — refusing rather than guessing a saturation rate",
                            model.substance
                        )
                    })?;
                Ok(SimShape {
                    lag_hours: 0.0,
                    absorption_half_life_hours: None,
                    rise_time_hours: Some(rise),
                    ceiling: param_f64(p, "ceiling").filter(|c| *c > 0.0),
                })
            }
            ModelClass::None => Err(format!(
                "{}: model class None is explicitly not modelled — no simulation (not implemented by design)",
                model.substance
            )),
        }
    }
}

/// Value at `t` of the class-specific closed form, superposed causally over
/// all doses at or before `t` (T216 for every class, not just decay).
/// `half_life_hours` drives decay/absorption/accumulation; `rise_hours`
/// drives the plateau band. Each is evaluated at its band edge.
fn curve_at(
    doses: &[DoseEvent],
    class: &crate::models::ModelClass,
    half_life_hours: f64,
    rise_hours: f64,
    shape: &SimShape,
    t: DateTime<Utc>,
) -> f64 {
    use crate::models::ModelClass;
    let mut total = 0.0;
    for d in doses {
        if d.timestamp > t {
            continue; // causality: the future contributes nothing
        }
        let dt = (t - d.timestamp).num_milliseconds() as f64 / 3_600_000.0;
        match class {
            ModelClass::Decay => {
                total += d.amount * 0.5f64.powf(dt / half_life_hours);
            }
            ModelClass::OralAbsorption => {
                let dt = (dt - shape.lag_hours).max(0.0);
                if dt == 0.0 {
                    continue;
                }
                // Bateman function: rises, peaks near the absorption time,
                // then the elimination half-life dominates the tail.
                let ke = LN2 / half_life_hours;
                let ka = LN2 / shape.absorption_half_life_hours.unwrap_or(1.0);
                let frac = if (ka - ke).abs() < 1e-9 {
                    // k_a ≈ k_e limit of the Bateman difference quotient
                    ka * dt * (-ke * dt).exp()
                } else {
                    (ka / (ka - ke)) * ((-ke * dt).exp() - (-ka * dt).exp())
                };
                total += d.amount * frac;
            }
            ModelClass::Plateau => {
                // v1 simplification (documented): each dose rises toward its
                // own amount and does not decay; the total is capped at the
                // body-pool ceiling when declared. Saturates, never returns
                // to baseline between doses (A1/AC-1).
                total += d.amount * (1.0 - 0.5f64.powf(dt / rise_hours));
            }
            ModelClass::Accumulation => {
                // Uptake into a slow pool: each dose rises toward its amount
                // at the elimination rate; repeated dosing trends toward a
                // steady state (A3/AC-2).
                total += d.amount * (1.0 - 0.5f64.powf(dt / half_life_hours));
            }
            ModelClass::None => unreachable!("None is refused in SimShape::resolve"),
        }
    }
    if matches!(class, ModelClass::Plateau) {
        if let Some(c) = shape.ceiling {
            total = total.min(c);
        }
    }
    total
}

/// Public sampling API (T220): dispatches over every implemented model
/// class. Band always (FR-033): the three lines are the curve evaluated at
/// the band's edge parameters; nothing here emits a lone line.
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
    use crate::models::ModelClass;
    // FR-036 runtime gate: no simulation runs on parameters whose origin is
    // unrecorded. Provenance is mandatory in the type; this makes an empty
    // (whitespace) value a refusal at the single simulation chokepoint, not
    // a display nicety.
    if model.provenance.trim().is_empty() {
        return Err(format!(
            "{}: refusing to simulate without provenance (FR-036) — every parameter used must record its origin",
            model.substance
        ));
    }
    let shape = SimShape::resolve(model)?;
    if end < start {
        return Err("series window end precedes start".into());
    }
    if !step_hours.is_finite() || step_hours <= 0.0 {
        return Err(format!(
            "step_hours {step_hours} is not a positive finite number"
        ));
    }
    // Band edges: half-life-driven classes use the catalog t½ band;
    // plateau is insensitive to t½, so its band rides on rise_time
    // (catalog range `rise_time_range: [lo, hi]` when present, else the
    // documented DEFAULT_VARIABILITY, flagged as an assumption).
    let (band_a, band_b, band_c, band_is_assumption) =
        if matches!(model.model_class, ModelClass::Plateau) {
            let rise = shape.rise_time_hours.unwrap_or(1.0);
            let range = model
                .params
                .get("rise_time_range")
                .and_then(|v| v.as_array())
                .filter(|a| a.len() == 2)
                .and_then(|a| Some((a[0].as_f64()?, a[1].as_f64()?)))
                .filter(|(lo, hi)| *lo > 0.0 && *hi >= *lo);
            match range {
                Some((lo, hi)) => (lo, rise, hi, false),
                None => (
                    rise * (1.0 - DEFAULT_VARIABILITY),
                    rise,
                    rise * (1.0 + DEFAULT_VARIABILITY),
                    true,
                ),
            }
        } else {
            band_half_lives(model)?
        };
    let plateau = matches!(model.model_class, ModelClass::Plateau);
    let rise_unused = shape.rise_time_hours.unwrap_or(1.0);

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
        let a = curve_at(
            doses,
            &model.model_class,
            band_a,
            if plateau { band_a } else { rise_unused },
            &shape,
            t,
        );
        let b = curve_at(
            doses,
            &model.model_class,
            band_b,
            rise_unused.max(0.0),
            &shape,
            t,
        );
        let c = curve_at(
            doses,
            &model.model_class,
            band_c,
            if plateau { band_c } else { rise_unused },
            &shape,
            t,
        );
        out.push(ActiveDosePoint {
            t,
            low: a.min(c),
            expected: b,
            high: a.max(c),
            unit: unit.clone(),
            band_is_assumption,
        });
    }
    Ok(out)
}

/// T219: pick a render window for a characteristic time (t½ for decay/
/// oral, rise time for plateau, t½ for accumulation) so long half-lives
/// neither truncate the curve nor explode the point count. Shows ~8
/// characteristic times (>99.6 % of the transition), clamped to 24 h…30 d,
/// sampled in ~200 steps (well under MAX_SERIES_POINTS).
pub fn recommended_window(characteristic_hours: f64) -> Result<(f64, f64), String> {
    if !characteristic_hours.is_finite() || characteristic_hours <= 0.0 {
        return Err(format!(
            "characteristic time must be finite and > 0 h, got {characteristic_hours}"
        ));
    }
    let window_hours = (8.0 * characteristic_hours).clamp(24.0, 720.0);
    let step_hours = window_hours / 200.0;
    Ok((window_hours, step_hours))
}

/// T222/T223 — verify the shared test-vector fixture (`tests/fixtures/
/// pk_vectors.json`, expectations computed independently in Python).
/// Single source of truth consumed by BOTH the native engine tests and the
/// wasm32 web tests: an empty return means this target reproduces every
/// recorded value within the documented tolerance (FR-038, SC-033).
pub fn check_vectors(json: &str) -> Vec<String> {
    use chrono::TimeZone;
    #[derive(serde::Deserialize)]
    struct Fix {
        tolerance: f64,
        step_hours: f64,
        vectors: Vec<Vector>,
    }
    #[derive(serde::Deserialize)]
    struct Vector {
        name: String,
        model: SubstanceModel,
        doses: Vec<RawDose>,
        checks: Vec<Chk>,
    }
    #[derive(serde::Deserialize)]
    struct RawDose {
        amount: f64,
        unit: String,
        offset_hours: f64,
    }
    #[derive(serde::Deserialize)]
    struct Chk {
        t_hours: f64,
        low: f64,
        expected: f64,
        high: f64,
    }

    let fix: Fix = match serde_json::from_str(json) {
        Ok(f) => f,
        Err(e) => return vec![format!("fixture parse error: {e}")],
    };
    let epoch = match Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return vec!["fixture epoch is not a single UTC instant".into()],
    };
    let mut failures = Vec::new();
    for v in &fix.vectors {
        let doses: Vec<DoseEvent> = v
            .doses
            .iter()
            .enumerate()
            .map(|(i, d)| DoseEvent {
                substance: v.model.substance.clone(),
                amount: d.amount,
                unit: d.unit.clone(),
                timestamp: epoch
                    + Duration::milliseconds((d.offset_hours * 3_600_000.0).round() as i64),
                entry_id: format!("{}#{i}", v.name),
            })
            .collect();
        let end = epoch + Duration::hours(192);
        let series = match sample_series(&v.model, &doses, epoch, end, fix.step_hours) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: sample_series failed: {e}", v.name));
                continue;
            }
        };
        for c in &v.checks {
            let idx = (c.t_hours / fix.step_hours).round() as usize;
            let Some(pt) = series.get(idx) else {
                failures.push(format!("{}: no sample at t={}h", v.name, c.t_hours));
                continue;
            };
            let off_h = (pt.t - epoch).num_milliseconds() as f64 / 3_600_000.0;
            if (off_h - c.t_hours).abs() > 1e-9 {
                failures.push(format!(
                    "{}: sample offset {off_h} != check t {}",
                    v.name, c.t_hours
                ));
                continue;
            }
            for (label, got, want) in [
                ("low", pt.low, c.low),
                ("expected", pt.expected, c.expected),
                ("high", pt.high, c.high),
            ] {
                if (got - want).abs() > fix.tolerance {
                    failures.push(format!(
                        "{}: t={}h {label}: got {got}, want {want} (tol {})",
                        v.name, c.t_hours, fix.tolerance
                    ));
                }
            }
        }
    }
    failures
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

    #[test]
    fn oral_absorption_peaks_then_halves_every_half_life() {
        // A3/AC caffeine shape: peak near absorption time, tail halves per t½.
        let mut m = model(Some(5.0), None);
        m.model_class = ModelClass::OralAbsorption;
        m.params = serde_json::json!({ "absorption_half_life_hours": 1.0 });
        let doses = vec![DoseEvent {
            substance: "Caf".into(),
            amount: 200.0,
            unit: "mg".into(),
            timestamp: t0(),
            entry_id: "e".into(),
        }];
        let series =
            sample_series(&m, &doses, t0(), t0() + Duration::hours(30), 0.5).expect("series");
        assert!(series[0].expected < 1e-9); // nothing absorbed at t = 0
        let peak = series
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.expected.partial_cmp(&b.1.expected).unwrap())
            .unwrap();
        // Bateman t_max = ln(ka/ke)/(ka-ke) ≈ 2.9 h for these parameters
        let peak_h = (peak.1.t - t0()).num_minutes() as f64 / 60.0;
        assert!((1.5..5.0).contains(&peak_h), "peak at {peak_h}h");
        // Late tail: absorption term is gone, so C(t+5)/C(t) = 0.5 exactly.
        let c = |h: i64| series[(h as usize) * 2].expected; // 0.5 h steps
        let ratio = c(29.0 as i64) / c(24);
        assert!((ratio - 0.5).abs() < 1e-4, "tail ratio {ratio}");
        assert!(series[40].low < series[40].expected && series[40].high > series[40].expected);
    }

    #[test]
    fn oral_absorption_without_absorption_param_is_refused() {
        let mut m = model(Some(5.0), None);
        m.model_class = ModelClass::OralAbsorption;
        let err = sample_series(
            &m,
            &[dose(100.0, t0())],
            t0(),
            t0() + Duration::hours(1),
            1.0,
        )
        .expect_err("must refuse");
        assert!(err.contains("absorption"), "{err}");
        assert!(err.contains("refusing"), "{err}");
    }

    #[test]
    fn plateau_saturates_and_never_returns_to_baseline() {
        // A1/AC-1 creatine shape: daily dosing saturates at the ceiling.
        let mut m = model(None, None);
        m.model_class = ModelClass::Plateau;
        m.params = serde_json::json!({ "rise_time_hours": 24.0, "ceiling": 25.0 });
        let doses: Vec<DoseEvent> = (0..7)
            .map(|d| DoseEvent {
                substance: "Cre".into(),
                amount: 5.0,
                unit: "g".into(),
                timestamp: t0() + Duration::days(d),
                entry_id: "e".into(),
            })
            .collect();
        let series =
            sample_series(&m, &doses, t0(), t0() + Duration::days(7), 6.0).expect("series");
        // Monotonic: never dips between doses.
        for w in series.windows(2) {
            assert!(
                w[1].expected >= w[0].expected - 1e-12,
                "dip at {:?}",
                w[1].t
            );
        }
        let last = series.last().unwrap();
        assert!(
            (last.expected - 25.0).abs() < 0.05,
            "ceiling hit: {}",
            last.expected
        );
        assert!(last.low <= last.expected && last.high >= last.expected);
        assert!(last.band_is_assumption); // no rise_time_range given
    }

    #[test]
    fn accumulation_rises_toward_steady_state() {
        // A3/AC-2 vitamin-D shape: daily dosing trends upward across a week.
        let mut m = model(Some(120.0), None); // 5-day pool half-life
        m.model_class = ModelClass::Accumulation;
        let doses: Vec<DoseEvent> = (0..7)
            .map(|d| DoseEvent {
                substance: "D3".into(),
                amount: 1000.0,
                unit: "iu".into(),
                timestamp: t0() + Duration::days(d),
                entry_id: "e".into(),
            })
            .collect();
        let series =
            sample_series(&m, &doses, t0(), t0() + Duration::days(7), 12.0).expect("series");
        for w in series.windows(2) {
            assert!(w[1].expected >= w[0].expected - 1e-12, "must trend up");
        }
        let day1 = series[2].expected;
        let day6 = series[12].expected;
        assert!(day6 > day1 * 4.0, "rising: day6 {day6} vs day1 {day1}");
        // Bounded: the pool converges, it never grows past total dosed.
        assert!(day6 < 7000.0, "must be bounded below total administered");
    }

    #[test]
    fn none_class_still_refuses_explicitly() {
        let mut m = model(Some(6.0), None);
        m.model_class = ModelClass::None;
        let err = sample_series(&m, &[dose(1.0, t0())], t0(), t0() + Duration::hours(1), 1.0)
            .expect_err("must refuse");
        assert!(err.contains("not modelled"), "{err}");
    }

    #[test]
    fn shared_vectors_match_python_expectations() {
        let fixture = include_str!("../tests/fixtures/pk_vectors.json");
        let failures = check_vectors(fixture);
        assert!(
            failures.is_empty(),
            "vector drift:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn t219_recommended_window_bounds_long_and_short_half_lives() {
        // caffeine-scale: 5 h t½ -> 40 h window
        let (w, step) = recommended_window(5.0).unwrap();
        assert!((w - 40.0).abs() < 1e-9);
        // vitamin-D-scale: 120 h t½ would want 960 h, clamped to 30 d
        let (w, step_long) = recommended_window(120.0).unwrap();
        assert!((w - 720.0).abs() < 1e-9);
        // never more than ~201 points, so sample_series can't hit the cap
        for ch in [0.01, 5.0, 120.0, 5000.0] {
            let (w, s) = recommended_window(ch).unwrap();
            assert!(w / s <= 200.5, "window {w} step {s} exceeds point budget");
        }
        assert!(recommended_window(0.0).is_err());
        assert!(recommended_window(f64::NAN).is_err());
        let _ = step;
        let _ = step_long;
    }

    #[test]
    fn t224_dst_boundary_is_invisible_to_the_sim() {
        // EU summer time ends 2026-10-25 01:00 UTC: Bucharest wall clock
        // repeats 02:00->03:00 as +03:00 -> +02:00. The engine stores UTC
        // instants only (FR: "internally always UTC"), so DST must be a
        // pure display concern. chrono-tz is not vendored offline, so this
        // tests the invariant directly with fixed offsets.
        use chrono::{FixedOffset, TimeZone};
        let epoch = Utc.with_ymd_and_hms(2026, 10, 24, 22, 0, 0).unwrap();
        let model = model(Some(6.0), None);
        let doses = vec![DoseEvent {
            substance: "VecDecay".into(),
            amount: 100.0,
            unit: "mg".into(),
            timestamp: epoch,
            entry_id: "dst#0".into(),
        }];
        // Same Bucharest wall time (03:30 local) before and after the flip:
        // 00:30Z at +03:00 and 01:30Z at +02:00 both read 03:30 local.
        let before = FixedOffset::east_opt(3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 25, 3, 30, 0)
            .unwrap();
        let after = FixedOffset::east_opt(2 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 25, 3, 30, 0)
            .unwrap();
        assert_eq!(before.to_utc(), epoch + Duration::minutes(150));
        assert_eq!(after.to_utc(), epoch + Duration::minutes(210));
        // identical wall clock, one real hour apart -> decay follows UTC,
        // not the repeated wall hour
        let c_before = onboard_at(&doses, 6.0, before.to_utc());
        let c_after = onboard_at(&doses, 6.0, after.to_utc());
        assert!(c_after < c_before, "decay must advance with UTC across DST");
        // sampled series stays exactly uniform in UTC across the flip
        let series =
            sample_series(&model, &doses, epoch, epoch + Duration::hours(48), 3.0).unwrap();
        for pair in series.windows(2) {
            assert_eq!(pair[1].t - pair[0].t, Duration::hours(3));
        }
    }
}

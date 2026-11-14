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
    let (nums_part, unit) = split_number_unit(base).ok_or_else(|| format!("'{raw}': no recognisable number+unit"))?;

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
    let mut audit = CatalogPkAudit { single: vec![], ranges: vec![], failures: vec![] };
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
        assert_eq!(parse_half_life("5-6 hours"), Ok(ParsedHalfLife::Range(5.0, 6.0)));
        assert_eq!(parse_half_life("5 - 6 h"), Ok(ParsedHalfLife::Range(5.0, 6.0)));
        // The alcohol form from the real catalog: range + annotation.
        assert_eq!(
            parse_half_life("~4-6h (zero-order ~7 g/h)"),
            Ok(ParsedHalfLife::Range(4.0, 6.0))
        );
    }

    #[test]
    fn rejects_unknown_forms_with_explicit_reasons() {
        for bad in ["unknown", "N/A", "varies", "", "~", "abc", "5 monkeys", "0h", "-3h", "5-6", "h5"] {
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
        assert_eq!(total, catalog.len(), "every item must land in exactly one bucket");

        // The one known failure in today's seed catalog: 9-Me-BC ("unknown").
        // If this set changes, the change must be deliberate — update with a
        // reason, never silently.
        let failure_names: Vec<&str> = audit.failures.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(failure_names, vec!["9-Me-BC"], "unexpected half-life parse failures: {failure_names:?}");

        // Spot-check the conversions that matter clinically.
        let caffeine = audit.single.iter().find(|(n, _)| n == "Caffeine").unwrap().1;
        assert_eq!(caffeine, 5.0);
        let estradiol = audit.single.iter().find(|(n, _)| n == "Estradiol Enanthate").unwrap().1;
        assert_eq!(estradiol, 192.0); // ~8d
        let alcohol = audit.ranges.iter().find(|(n, ..)| n == "Alcohol").unwrap();
        assert_eq!((alcohol.1, alcohol.2), (4.0, 6.0));
    }
}

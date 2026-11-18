//! Verified depot-release parameters for `ModelClass::ImDepotAbsorption
//!` models (#183).
//!
//! Sister table to [`crate::pk_absorption`], deliberately separate: an
//! IM/SC depot releases drug at the injection site, and the rate constant
//! measured there (depot residence) is a different physical quantity than
//! gut absorption. Values must come from studies that measured the
//! injection itself — never borrowed from the oral table, never derived
//! from oral bioavailability studies. Flip-flop profiles (terminal slope
//! reflecting release, not metabolism) are legitimate here and expected;
//! the flag is recording which limb the reported half-life describes.
//!
//! Empty until a primary injection study with verbatim rate constants is
//! source-verified. Estradiol Enanthate carries Kabs = 1.5050 d⁻¹ /
//! K = 0.12445 d⁻¹ figures in its assignment rationale but the PMID
//! trail must be re-verified against the primary record before entry.

/// One verified depot-release research entry.
#[derive(Debug, Clone, PartialEq)]
pub struct DepotResearch {
    /// Exact catalog display name.
    pub substance: &'static str,
    /// First-order depot-release half-life in hours.
    pub absorption_half_life_hours: f64,
    /// Citation + derivation + caveats, as one string.
    pub provenance: &'static str,
}

/// SEED: the single producer of depot-release parameters. Keep sorted
/// by substance name; every row must re-solve from its cited inputs in
/// the reproduction test below.
pub const DEPOT_RESEARCH: &[DepotResearch] = &[
    // Intentionally empty at landing (#183): the class exists so
    // Estradiol Enanthate stops pretending to be an oral model, but no
    // injection-study rate constant has passed source verification yet.
];

/// Case-insensitive lookup by catalog name.
pub fn lookup(name: &str) -> Option<&'static DepotResearch> {
    let needle = name.trim().to_lowercase();
    DEPOT_RESEARCH
        .iter()
        .find(|r| r.substance.to_lowercase() == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::seed_catalog;
    use crate::models::ModelClass;
    use crate::pharmacokinetics::{DoseEvent, sample_series};
    use crate::pk_assignments::build_models;
    use chrono::{TimeZone, Utc};

    fn dose(substance: &str, amount: f64) -> DoseEvent {
        DoseEvent {
            substance: substance.into(),
            amount,
            unit: "mg".into(),
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            entry_id: "test".into(),
        }
    }

    #[test]
    fn estradiol_enanthate_is_a_depot_not_an_oral_model() {
        // #183: reassignment means the honest refusal message now names
        // the depot, not the gut.
        let items = seed_catalog();
        let models = build_models(&items);
        let m = models
            .iter()
            .find(|m| m.substance == "Estradiol Enanthate")
            .expect("enanthate model built");
        assert_eq!(m.model_class, ModelClass::ImDepotAbsorption);
    }

    #[test]
    fn depot_substances_without_release_research_still_refuse() {
        // Self-maintaining like the oral twin: whatever ImDepotAbsorption
        // substance lacks a DEPOT_RESEARCH row must refuse, and the
        // refusal must say depot so nobody confuses it with an oral gap.
        let items = seed_catalog();
        let models = build_models(&items);
        let mut checked = 0;
        for m in models
            .iter()
            .filter(|m| m.model_class == ModelClass::ImDepotAbsorption)
            .filter(|m| lookup(&m.substance).is_none())
        {
            let d = dose(&m.substance, 10.0);
            let start = d.timestamp;
            let err = sample_series(m, &[d], start, start + chrono::Duration::hours(24), 1.0)
                .expect_err("depot class without a release parameter must refuse");
            assert!(
                err.to_string().contains("depot"),
                "refusal must name the depot: {err}"
            );
            checked += 1;
        }
        assert!(checked > 0, "the depot class must have members to police");
    }
}

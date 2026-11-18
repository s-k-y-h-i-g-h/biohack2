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
//! source-verified — the Estradiol Enanthate entry below was the first
//! through that gate (pubmed:3814225 states both limbs outright).

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
pub const DEPOT_RESEARCH: &[DepotResearch] = &[DepotResearch {
    substance: "Estradiol Enanthate",
    // Primary study verified verbatim against the PubMed abstract
    // (pubmed:3814225 — Wiemeyer et al. 1986, Arzneimittelforschung
    // 36(11):1674): healthy menopausic women, 10 mg i.m. estradiol
    // enantate, serum E2 by RIA on days 1-31. The study states the
    // rate constants outright:
    //     Kabs = 1.5050/d  -> t1/2,abs = 0.460563 d = 11.0535 h
    //     K    = 0.12445/d -> t1/2,elim = 5.5697 d = 133.7 h
    //         (matches the catalog half_life_hours 133.7 exactly)
    // Kabs/K = 12.1: elimination, not release, sets the terminal
    // slope — the old 'flip-flop' label on this substance was wrong
    // and the rationale has been corrected. Bateman peak 1.81 d,
    // consistent with the classic 1-3 day E2 peak after IM esters.
    // Caveats: single primary study, n not stated in abstract,
    // daily sampling grid puts tmax resolution at +/- 0.5 d; the
    // study computed both limbs itself — no re-derivation by us
    // beyond unit conversion (day -> hour).
    absorption_half_life_hours: 11.0535,
    provenance: "pubmed:3814225 — Wiemeyer 1986, Arzneimittelforschung; Kabs 1.5050/d stated verbatim -> t1/2,abs = ln2/1.5050 d x 24 = 11.0535 h; K 0.12445/d reproduces catalog t1/2 133.7 h; ka/ke = 12.1 (elimination-limited tail, NOT flip-flop); single study, daily sampling",
}];

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
    fn estradiol_enanthate_depot_release_reproduces_from_cited_constant() {
        // t1/2,abs = ln2 / Kabs from the study's own stated constant.
        let dep = lookup("Estradiol Enanthate").expect("entry exists");
        let solved = 2f64.ln() / 1.5050 * 24.0; // day -> hour
        assert!(
            (dep.absorption_half_life_hours - solved).abs() < 0.0005,
            "table says {} but cited Kabs derives {solved:.4}",
            dep.absorption_half_life_hours
        );
    }

    #[test]
    fn enanthate_curve_peaks_near_the_classic_one_to_three_day_e2_peak() {
        let items = seed_catalog();
        let models = build_models(&items);
        let m = models
            .iter()
            .find(|m| m.substance == "Estradiol Enanthate")
            .unwrap();
        let d = dose("Estradiol Enanthate", 10.0);
        let start = d.timestamp;
        let series = sample_series(
            m,
            &[d],
            start,
            start + chrono::Duration::hours(24 * 8),
            2.0, // 2-hour grid: peak resolution matches daily sampling
        )
        .expect("depot release param now exists — enanthate must simulate");
        let peak = series
            .iter()
            .max_by(|a, b| a.expected.total_cmp(&b.expected))
            .unwrap();
        let peak_h = (peak.t - start).num_seconds() as f64 / 3600.0;
        assert!(
            (40.0..=48.0).contains(&peak_h),
            "peak at t={peak_h:.1} h, Bateman says 43.3 h (1.8 d)"
        );
    }

    #[test]
    fn depot_substances_without_release_research_still_refuse() {
        // Self-maintaining like the oral twin: whatever ImDepotAbsorption
        // substance lacks a DEPOT_RESEARCH row must refuse, and the
        // refusal must say depot so nobody confuses it with an oral gap.
        let items = seed_catalog();
        let models = build_models(&items);
        let mut members = 0;
        for m in models
            .iter()
            .filter(|m| m.model_class == ModelClass::ImDepotAbsorption)
        {
            members += 1;
            if lookup(&m.substance).is_some() {
                continue; // researched members are policed by the other tests
            }
            let d = dose(&m.substance, 10.0);
            let start = d.timestamp;
            let err = sample_series(m, &[d], start, start + chrono::Duration::hours(24), 1.0)
                .expect_err("depot class without a release parameter must refuse");
            assert!(
                err.to_string().contains("depot"),
                "refusal must name the depot: {err}"
            );
        }
        assert!(members > 0, "the depot class must have members to police");
    }
}

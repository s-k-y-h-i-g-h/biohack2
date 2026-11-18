//! Resorption-phase parameters for `OralAbsorption` models (#182).
//!
//! Phase 6 assigned many substances to `ModelClass::OralAbsorption`, but
//! nothing in the crate emitted the `absorption_half_life_hours` /
//! `ka_per_hour` key that `pharmacokinetics::SimShape` requires — so every
//! such model refused to simulate. This table is the single producer.
//!
//! Provenance rules (same rigor as `pk_parameters`):
//! - Every value carries a citation, or an explicit derivation formula
//!   with its inputs and their citations. A derived value must be
//!   reproducible from the cited numbers alone.
//! - A substance with no defensible absorption value stays absent here:
//!   the simulation then refuses rather than guessing (FR-036). That is
//!   the correct state, not a TODO to paper over.

/// One substance's resorption-phase parameter.
pub struct AbsorptionResearch {
    pub substance: &'static str,
    /// First-order absorption half-life, hours.
    pub absorption_half_life_hours: f64,
    /// Where the number comes from, audit-style.
    pub provenance: &'static str,
}

pub const RESEARCH: &[AbsorptionResearch] = &[AbsorptionResearch {
    substance: "Melatonin",
    // Derived from numbers already in pk_parameters::RESEARCH (Harpsoe
    // 2015 pooled values): one-compartment Bateman time-to-peak,
    //     tmax = ln(ka/ke) / (ka - ke),  ke = ln2 / 0.75 h
    // solved at tmax = 50/60 h  ->  ka = 1.526/h  ->  t½,abs = ln2/ka
    //     = 0.454 h  (rounded 0.45).
    // The formula is dose- and bioavailability-independent (both cancel),
    // so the ~15% oral F does not bias it. Sensitivity band from the
    // review's reported spread: Tmax 40–60 min -> t½,abs 0.30–0.64 h.
    absorption_half_life_hours: 0.45,
    provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at pooled oral Tmax 50 min with ke from T1/2 45 min (pubmed:26008214 — Harpsoe et al. 2015, Eur J Clin Pharmacol); Tmax 40-60 min -> 0.30-0.64 h band",
}];

pub fn lookup(name: &str) -> Option<&'static AbsorptionResearch> {
    RESEARCH
        .iter()
        .find(|a| a.substance.eq_ignore_ascii_case(name))
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
    fn lookup_is_case_insensitive() {
        assert!(lookup("melatonin").is_some());
        assert!(lookup("Melatonin").is_some());
        assert!(lookup("Caffeine").is_none()); // until researched
    }

    #[test]
    fn derived_melatonin_absorption_reproduces_from_cited_numbers() {
        // Independent re-solve of the Bateman equation against the cited
        // inputs: if someone edits the table value without editing the
        // derivation, this fails.
        let ln2 = 2f64.ln();
        let ke = ln2 / 0.75;
        let target = 50f64 / 60.0;
        let tmax = |ka: f64| (ka / ke).ln() / (ka - ke);
        let (mut lo, mut hi) = (ke * 1.001, 50.0);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if tmax(mid) > target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t_half_abs = ln2 / ((lo + hi) / 2.0);
        let table = lookup("Melatonin").unwrap();
        assert!(
            (table.absorption_half_life_hours - t_half_abs).abs() < 0.01,
            "table says {} but cited inputs derive {t_half_abs:.3}",
            table.absorption_half_life_hours
        );
    }

    #[test]
    fn melatonin_now_simulates_with_peak_near_labeled_tmax() {
        let items = seed_catalog();
        let models = build_models(&items);
        let mel = models
            .iter()
            .find(|m| m.substance == "Melatonin")
            .expect("melatonin model built");
        assert_eq!(mel.model_class, ModelClass::OralAbsorption);
        assert!(
            mel.provenance.contains("26008214"),
            "absorption provenance must chain to the review: {}",
            mel.provenance
        );

        let d = dose("Melatonin", 2.0);
        let start = d.timestamp;
        let series = sample_series(mel, &[d], start, start + chrono::Duration::hours(6), 0.25)
            .expect("melatonin must simulate once absorption params exist");

        let peak = series
            .iter()
            .max_by(|a, b| a.expected.total_cmp(&b.expected))
            .unwrap();
        let peak_h = (peak.t - start).num_minutes() as f64 / 60.0;
        // Labeled Tmax 0.75–0.83 h; grid is 15 min, allow ±30 min.
        assert!(
            (0.4..=1.35).contains(&peak_h),
            "peak at t={peak_h:.2} h, expected near labeled Tmax ~0.75-0.83 h"
        );
        assert!(peak.expected > 0.0);
        // Sanity: t0 starts at zero (absorption phase, unlike instant Decay).
        assert!(
            series[0].expected < 0.001 * peak.expected,
            "oral model must start at ~0"
        );
    }

    #[test]
    fn substances_without_absorption_research_still_refuse() {
        // #182 discipline: adding the table must not silently unblock
        // unresearched OralAbsorption substances with guessed numbers.
        let items = seed_catalog();
        let models = build_models(&items);
        let caf = models
            .iter()
            .find(|m| m.substance == "Caffeine")
            .expect("caffeine model built");
        assert_eq!(caf.model_class, ModelClass::OralAbsorption);
        let d = dose("Caffeine", 200.0);
        let t0 = d.timestamp;
        let err = sample_series(caf, &[d], t0, t0 + chrono::Duration::hours(6), 1.0)
            .expect_err("caffeine has no absorption param yet — must stay an honest refusal");
        assert!(err.contains("absorption"), "{err}");
    }
}

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

pub const RESEARCH: &[AbsorptionResearch] = &[
    AbsorptionResearch {
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
    },
    AbsorptionResearch {
        substance: "Caffeine",
        // Same method, same provenance discipline: Blanchard & Sawers 1983
        // measured Tmax 29.8 +/- 8.1 min in healthy adults (already cited in
        // pk_parameters params_note); ke from the engine's 5.0 h elimination.
        //     ka = 8.40/h  ->  t½,abs = ln2/ka = 0.0815 h (4.9 min)
        // Band from the SD on Tmax (21.7-37.9 min): 0.055-0.113 h.
        // Consistent with Magkos 2005 (pubmed:16371327): "rapidly and
        // completely absorbed"; absolute F ~108% so no first-pass distortion.
        absorption_half_life_hours: 0.0815,
        provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at measured Tmax 29.8 min (pubmed:6832208 — Blanchard & Sawers 1983, Eur J Clin Pharmacol) with ke from T1/2 5.0 h (pubmed:27100333); Tmax 21.7-37.9 min -> 0.055-0.113 h band",
    },
    AbsorptionResearch {
        substance: "Ashwagandha",
        // Full-text-confirmed (PMC10746415): total withanolides after WS-35,
        // Tmax 2.277 +/- 0.09 h, t½ 9.756 +/- 0.517 h — the same study row
        // the engine's 9.76 h half-life came from. ke = ln2/9.756 =
        // 0.0710/h;  ka = 1.371/h  ->  t½,abs = 0.5056 h.
        // ka >> ke so no flip-flop: the 9.76 h is genuinely elimination.
        // Band from the SD on Tmax (2.187-2.367 h): 0.478-0.534 h.
        // Caveat kept honest: "total withanolides" is an LC-MS/MS mixture
        // measure; per-analyte Tmax (withanolide A 1.83, withaferin A 2.28,
        // withanoside IV 1.76) cluster within the band, so the
        // one-compartment approximation is defensible for the lumped entry.
        absorption_half_life_hours: 0.5056,
        provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at measured Tmax 2.277 h (pubmed:38144272 — Kim 2023, Heliyon, PMC10746415 full text) with ke from WS-35 total-withanolide T1/2 9.756 h (same study); Tmax 2.187-2.367 h -> 0.478-0.534 h band",
    },
    AbsorptionResearch {
        substance: "Berberine",
        // Full-text-confirmed (PMC8979756): berberine under FASTED
        // conditions, n=12 healthy males, UI026 syrup (Coptis extract),
        // Tmax median 1.00 h (IQR 1.00-1.33), t1/2 5.77 h. ke kept as the
        // engine's own 5.0 h (same cited table row family) so absorption
        // and elimination stay internally consistent in the model.
        //     ka = 3.312/h  ->  t½,abs = 0.2093 h
        // Band from the Tmax IQR upper bound (1.33 h): 0.209-0.311 h.
        // Honest caveats: single small food-effect study; NCA with sparse
        // sampling (Tmax quantised to the 1.00/1.33 h grid points);
        // absolute F <1% - irrelevant here, Bateman tmax is F-independent;
        // ka 3.31/h >> ke 0.139/h so no flip-flop.
        absorption_half_life_hours: 0.2093,
        provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at fasted Tmax median 1.00 h (pubmed:35419311 — Park 2022, Transl Clin Pharmacol, PMC8979756 full text) with ke from engine T1/2 5.0 h (same source); Tmax IQR to 1.33 h -> 0.209-0.311 h band",
    },
    AbsorptionResearch {
        substance: "Reishi Mushroom",
        // Full-text-confirmed (PMC5055913): ganoderic acid A (marker
        // compound) after Ling Zhi MG2FB-WE preparation, n=12 healthy
        // males, Reishi-only phase of a crossover: Tmax 0.54 ± 0.10 h,
        // t1/2 0.66 ± 0.33 h — the exact rows the catalog cites.
        //     ke = ln2/0.66 = 1.0502/h, ka = 2.984/h
        //     t½,abs = 0.2323 h
        // Band 0.23-0.28 h across Tmax ± SD x t1/2 ± SD corners.
        // Honest caveats: ka/ke is only ~2.8 (one-compartment oral
        // approximation is usable but the absorption/elimination limbs
        // are not cleanly separated — at the fast-t1/2 corner the ratio
        // drops to ~1.2); marker-compound kinetics for a variable
        // extract; t1/2 SD is half its mean.
        absorption_half_life_hours: 0.2323,
        provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at measured Tmax 0.54 h (pubmed:27747224 — Tawasri 2016, Biomed Res Int, PMC5055913 full text) with ke from ga-T1/2 0.66 h (same study); ka/ke ~2.8 modest separation, band 0.23-0.28 h",
    },
    AbsorptionResearch {
        substance: "NAC (N-Acetyl Cysteine)",
        // Full-text-confirmed (PMC10376274): Cetylev 11 g oral solution
        // (dissolved effervescent tablets), n=29 healthy adults — the
        // label study the review summarizes: Cmax 26.5 (CV 29%) ug/mL,
        // AUCinf 186, Tmax median 2 h (range 1-3.5 h). IV terminal
        // half-life in the same review: 5.6 h; the engine runs 6.25 h
        // (Holdiness 1991), and ke uses the engine value for internal
        // consistency of the model limbs.
        //     ke = ln2/6.25 = 0.1109/h, ka = 1.367/h
        //     t½,abs = 0.5072 h   (ka/ke = 12.3, clean separation)
        // Band across the observed Tmax range 1-3.5 h: 0.19-1.18 h.
        // Honest caveats: 11 g loading dose (APAP-overdose strength)
        // far above 600-1200 mg supplement use; effervescent solution,
        // so solid-tablet disintegration is not represented; sparse-
        // sampling median. First-order absorption is still supported
        // by the clean ka/ke separation.
        absorption_half_life_hours: 0.5072,
        provenance: "derived: one-compartment Bateman tmax=ln(ka/ke)/(ka-ke) solved at median Tmax 2 h (range 1-3.5; pubmed:37453798 review, PMC10376274 full text, Cetylev n=29 label study) with ke from engine T1/2 6.25 h (Holdiness 1991); band 0.19-1.18 h",
    },
];

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
        assert!(lookup("Caffeine").is_some());
        assert!(lookup("ashwagandha").is_some());
        assert!(lookup("berberine").is_some());
        assert!(lookup("reishi mushroom").is_some());
        assert!(lookup("nac (n-acetyl cysteine)").is_some());
        assert!(lookup("9-Me-BC").is_none()); // until researched
    }

    fn solve_bateman_t_half_abs(ke_per_hour: f64, tmax_hours: f64) -> f64 {
        let tmax = |ka: f64| (ka / ke_per_hour).ln() / (ka - ke_per_hour);
        let (mut lo, mut hi) = (ke_per_hour * 1.001, 200.0);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if tmax(mid) > tmax_hours {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        2f64.ln() / ((lo + hi) / 2.0)
    }

    #[test]
    fn derived_absorption_values_reproduce_from_cited_numbers() {
        // Independent re-solve of the Bateman equation against each entry's
        // cited inputs: if someone edits a table value without editing the
        // derivation, this fails.
        let ln2 = 2f64.ln();
        let mel = lookup("Melatonin").unwrap();
        let mel_solved = solve_bateman_t_half_abs(ln2 / 0.75, 50.0 / 60.0);
        assert!(
            (mel.absorption_half_life_hours - mel_solved).abs() < 0.01,
            "melatonin table says {} but cited inputs derive {mel_solved:.3}",
            mel.absorption_half_life_hours
        );
        let caf = lookup("Caffeine").unwrap();
        let caf_solved = solve_bateman_t_half_abs(ln2 / 5.0, 29.8 / 60.0);
        assert!(
            (caf.absorption_half_life_hours - caf_solved).abs() < 0.005,
            "caffeine table says {} but cited inputs derive {caf_solved:.4}",
            caf.absorption_half_life_hours
        );
        let ash = lookup("Ashwagandha").unwrap();
        let ash_solved = solve_bateman_t_half_abs(ln2 / 9.756, 2.277);
        assert!(
            (ash.absorption_half_life_hours - ash_solved).abs() < 0.005,
            "ashwagandha table says {} but cited inputs derive {ash_solved:.4}",
            ash.absorption_half_life_hours
        );
        let ber = lookup("Berberine").unwrap();
        let ber_solved = solve_bateman_t_half_abs(ln2 / 5.0, 1.00);
        assert!(
            (ber.absorption_half_life_hours - ber_solved).abs() < 0.005,
            "berberine table says {} but cited inputs derive {ber_solved:.4}",
            ber.absorption_half_life_hours
        );
        let rei = lookup("Reishi Mushroom").unwrap();
        let rei_solved = solve_bateman_t_half_abs(ln2 / 0.66, 0.54);
        assert!(
            (rei.absorption_half_life_hours - rei_solved).abs() < 0.005,
            "reishi table says {} but cited inputs derive {rei_solved:.4}",
            rei.absorption_half_life_hours
        );
        let nac = lookup("NAC (N-Acetyl Cysteine)").unwrap();
        let nac_solved = solve_bateman_t_half_abs(ln2 / 6.25, 2.0);
        assert!(
            (nac.absorption_half_life_hours - nac_solved).abs() < 0.005,
            "nac table says {} but cited inputs derive {nac_solved:.4}",
            nac.absorption_half_life_hours
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
        // #182 discipline: adding table entries must not silently unblock
        // unresearched OralAbsorption substances with guessed numbers.
        // Self-maintaining: picks whatever OralAbsorption model currently
        // has no table row and asserts it still refuses honestly.
        let items = seed_catalog();
        let models = build_models(&items);
        let un = models
            .iter()
            .find(|m| m.model_class == ModelClass::OralAbsorption && lookup(&m.substance).is_none())
            .expect("catalog still contains OralAbsorption substances with no absorption research");
        let d = dose(&un.substance, 500.0);
        let t0 = d.timestamp;
        let err = sample_series(un, &[d], t0, t0 + chrono::Duration::hours(6), 1.0)
            .expect_err("unresearched OralAbsorption substance must stay an honest refusal");
        assert!(err.contains("absorption"), "{err}");
    }

    #[test]
    fn caffeine_curve_peaks_at_measured_tmax() {
        // Biovalidation: with t½,abs derived from Blanchard & Sawers Tmax
        // and the 5 h elimination, the simulated peak must land at the
        // measured 29.8 min — on a 5-min grid.
        let items = seed_catalog();
        let models = build_models(&items);
        let caf = models
            .iter()
            .find(|m| m.substance == "Caffeine")
            .expect("caffeine model built");
        let d = dose("Caffeine", 200.0);
        let start = d.timestamp;
        let series = sample_series(
            caf,
            &[d],
            start,
            start + chrono::Duration::hours(6),
            1.0 / 12.0,
        )
        .expect("caffeine must simulate now that an absorption param exists");
        let peak = series
            .iter()
            .max_by(|a, b| a.expected.total_cmp(&b.expected))
            .unwrap();
        let peak_h = (peak.t - start).num_seconds() as f64 / 3600.0;
        assert!(
            (0.35..=0.67).contains(&peak_h),
            "peak at t={peak_h:.3} h, measured Tmax was 0.497 h (29.8 min)"
        );
    }

    #[test]
    fn reishi_curve_peaks_near_marker_tmax() {
        // Ganoderic acid A peaked at 0.54 ± 0.10 h in the Reishi-only
        // phase (PMC5055913). With ka 2.98/h + ke 1.05/h the Bateman
        // hump must peak there on a 5-min grid.
        let items = seed_catalog();
        let models = build_models(&items);
        let r = models
            .iter()
            .find(|m| m.substance == "Reishi Mushroom")
            .expect("reishi model built");
        let d = dose("Reishi Mushroom", 1.0);
        let start = d.timestamp;
        let series = sample_series(
            r,
            &[d],
            start,
            start + chrono::Duration::hours(6),
            1.0 / 12.0,
        )
        .expect("reishi must simulate now that an absorption param exists");
        let peak = series
            .iter()
            .max_by(|a, b| a.expected.total_cmp(&b.expected))
            .unwrap();
        let peak_h = (peak.t - start).num_seconds() as f64 / 3600.0;
        assert!(
            (0.41..=0.71).contains(&peak_h),
            "peak at t={peak_h:.3} h, measured Tmax was 0.54 h"
        );
    }

    #[test]
    fn nac_curve_peaks_at_label_tmax() {
        // The Cetylev label study peaked at median 2 h (PMC10376274).
        // Bateman with ka 1.367 + ke 0.111 must peak there.
        let items = seed_catalog();
        let models = build_models(&items);
        let r = models
            .iter()
            .find(|m| m.substance == "NAC (N-Acetyl Cysteine)")
            .expect("nac model built");
        let d = dose("NAC (N-Acetyl Cysteine)", 600.0);
        let start = d.timestamp;
        let series = sample_series(
            r,
            &[d],
            start,
            start + chrono::Duration::hours(24),
            1.0 / 12.0,
        )
        .expect("nac must simulate now that an absorption param exists");
        let peak = series
            .iter()
            .max_by(|a, b| a.expected.total_cmp(&b.expected))
            .unwrap();
        let peak_h = (peak.t - start).num_seconds() as f64 / 3600.0;
        assert!(
            (1.91..=2.09).contains(&peak_h),
            "peak at t={peak_h:.3} h, label Tmax was 2 h"
        );
    }
}

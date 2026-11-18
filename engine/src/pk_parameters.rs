//! Reference PK parameters with per-value citations (spec 003 Phase 6,
//! T251–T256).
//!
//! `SEED_MODEL_CLASSES` (pk_assignments) answers *how* a substance is
//! modelled. This table answers *with which numbers, and where they came
//! from*. Every entry carries a citation string per FR-036: a parameter
//! without a source is a guess wearing a lab coat, and guesses do not ship.
//!
//! Substances with no defensible human parameter set are NOT listed here;
//! they are assigned `ModelClass::None` with a rationale in
//! `SEED_MODEL_CLASSES` (T255, FR-040 — absence is a deliverable).
//!
//! Versioning (T257): changing any number here is a catalog-parameter
//! change. Bump `catalog::CATALOG_VERSION` in the same commit so
//! `SubstanceModel::catalog_version` staleness detection (FR-006) fires
//! for downstream consumers holding old rows.

use serde::{Deserialize, Serialize};

/// One researched parameter set. Strings are `&'static str` by design:
/// this table is compiled-in reference data, reviewed like code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameterized {
    /// Catalog item name, verbatim (must match `seed_catalog()` exactly —
    /// the coverage test enforces it).
    pub name: &'static str,
    /// Point-estimate elimination half-life, hours.
    pub half_life_hours: Option<f64>,
    /// Literature low-high range, hours (FR-033 band, endpoints preserved).
    pub half_life_range: Option<(f64, f64)>,
    /// Citation for the point value (`half_life_hours`), e.g.
    /// `"pubmed:20863248 — Blanchard 1986, J Clin Pharmacol"`. Empty only
    /// when `half_life_hours` is None.
    pub citation: &'static str,
    /// Citation(s) for the range endpoints when they differ from the point
    /// value's source; free text, one per line.
    pub range_citation: &'static str,
    /// Class-relevant extras with their citations inline (t_max,
    /// steady-state time, saturation ceiling, depot duration…). May be
    /// empty; becomes `SubstanceModel::params` under a `"notes"` key.
    pub params_note: &'static str,
}

/// Compiled-in reference table. Populated by the Phase 6 research build-out
/// (T251–T254); every entry cited per value (T256).
pub const SEED_PK_PARAMETERS: &[Parameterized] = &[
    Parameterized {
        name: "Vitamin D3",
        half_life_hours: Some(362.4),
        half_life_range: Some((307.2, 374.4)),
        citation: "pubmed:24885631 — Jones et al. 2014, J Clin Endocrinol Metab",
        range_citation: "pubmed:24885631 — Jones et al. 2014, J Clin Endocrinol Metab (25(OH)D2 half-life 13.9 d; DBP-genotype dependence)",
        params_note: "Half-life is of circulating 25(OH)D3, not parent cholecalciferol: 15.1 +/- 3.1 d overall, 14.7 d Gambia vs 15.6 d UK; 25(OH)D2 13.9 +/- 2.6 d; half-life scales with vitamin-D-binding-protein concentration (pubmed:24885631 — Jones et al. 2014, J Clin Endocrinol Metab). Steady state of the 25(OH)D pool takes ~2-3 months of daily dosing.",
    },
    Parameterized {
        name: "Vitamin K2",
        half_life_hours: Some(72.0),
        half_life_range: Some((72.0, 144.0)),
        citation: "pubmed:32575901 — Akbulut et al. 2020, Nutrients",
        range_citation: "pubmed:17158229 — Schurgers et al. 2007, Blood (MK-7 accumulates 7-8-fold vs MK-4 during prolonged intake)",
        params_note: "MK-7 circulates up to 144 h; 10-fold higher postprandial serum levels than vitamin K1, absorbed within 4 h (pubmed:32575901 — Akbulut et al. 2020, Nutrients); prolonged MK-7 intake drives serum accumulation to 7-8x baseline with much more stable levels than MK-4 (pubmed:17158229 — Schurgers et al. 2007, Blood).",
    },
    Parameterized {
        name: "B-Complex",
        half_life_hours: Some(1.5),
        half_life_range: Some((1.0, 2.0)),
        citation: "pubmed:29629048 — Talebian 2018, Electron Physician",
        range_citation: "pubmed:8604671 — Zempleni 1996, Am J Clin Nutr",
        params_note: "Representative member: riboflavin, plasma half-life ~1-2 h (pubmed:29629048 — Talebian 2018, Electron Physician). Absorption half-life 1.1 h with two-compartment kinetics; carrier-mediated absorption saturates near 27 mg absorbed per dose and ~50% of plasma clearance is urinary (pubmed:8604671 — Zempleni 1996, Am J Clin Nutr).",
    },
    Parameterized {
        name: "Magnesium Glycinate",
        half_life_hours: Some(6.7),
        half_life_range: None,
        citation: "pubmed:32642964 — da Costa 2020, Drugs R D",
        range_citation: "",
        params_note: "Proxy half-life computed from CL 1.38 L/h and V 13.3 L in women with pre-eclampsia; creatinine and body-weight covariates shift Mg handling vs healthy oral dosing (pubmed:32642964 — da Costa 2020, Drugs R D).",
    },
    Parameterized {
        name: "Zinc",
        half_life_hours: Some(4488.0),
        half_life_range: Some((2688.0, 6288.0)),
        citation: "pubmed:3565500 — Muirhead 1986, Am J Nephrol",
        range_citation: "pubmed:10612077 — Watson 1999, J Trace Elem Med Biol",
        params_note: "Half-life is whole-body 65Zn retention in healthy controls; range = mean +/- 1 SD (112-262 days; pubmed:3565500 — Muirhead 1986, Am J Nephrol); meal-zinc absorption ~20 +/- 5% with fractional absorption falling as intake rises (pubmed:10612077 — Watson 1999, J Trace Elem Med Biol).",
    },
    Parameterized {
        name: "Boron",
        half_life_hours: Some(21.0),
        half_life_range: Some((16.1, 25.9)),
        citation: "pubmed:6732506 — Jansen 1984, Arch Toxicol",
        range_citation: "pubmed:10050922 — Samman 1998, Biol Trace Elem Res",
        params_note: "t1/2-beta 21.0 +/- 4.9 h (mean +/- SD) after 562-611 mg boric acid by 20-min IV infusion; 98.7 +/- 9.1% of dose recovered in urine by 120 h, i.e. near-quantitative renal first-order elimination (pubmed:6732506 — Jansen 1984, Arch Toxicol); oral 10-mg dose shows 84% urinary recovery (pubmed:10050922 — Samman 1998, Biol Trace Elem Res).",
    },
    Parameterized {
        name: "Omega-3 Fish Oil",
        half_life_hours: Some(45.0),
        half_life_range: Some((45.0, 66.0)),
        citation: "pubmed:39299684 — Schmieta et al. 2025, Lipids",
        range_citation: "pubmed:41956323 — Symington et al. 2026, Am J Clin Nutr (whole-body n-3 PUFA turnover half-lives: EPA 3.4 +/- 2.7 d, n-3 DPA 6.4 +/- 5.3 d, DHA 6.3 +/- 8.9 d)",
        params_note: "Point/range are single-dose plasma half-lives of the acid forms: EPA 45 +/- 8 h, DHA 66 +/- 12 h after 2.2-2.3 g ethyl esters (pubmed:39299684 — Schmieta et al. 2025, Lipids). Tissue (delta-13C traced) half-life is days, not hours: EPA 3.4 d, DHA 6.3 d (pubmed:41956323 — Symington et al. 2026, Am J Clin Nutr).",
    },
    Parameterized {
        name: "L-Theanine",
        half_life_hours: Some(1.2),
        half_life_range: Some((1.2, 1.8)),
        citation: "pubmed:23096008 — Scheid et al. 2012, The Journal of Nutrition",
        range_citation: "pubmed:41845544 — Yamaura et al. 2026, Journal of Pharmaceutical Health Care and Sciences",
        params_note: "100 mg capsule: Tmax 0.8 h, Cmax 24.3 umol/L, terminal t1/2 1.2 h (pubmed:23096008 — Scheid et al. 2012, J Nutr, terminal t1/2 as cited in pubmed:41845544); 300 mg: Tmax 45 min, t1/2 1.8 h (pubmed:41845544 — Yamaura et al. 2026, J Pharm Health Care Sci)",
    },
    Parameterized {
        name: "Piracetam",
        half_life_hours: Some(5.8),
        half_life_range: Some((5.1, 6.9)),
        citation: "pubmed:41851591 — Yan 2026, Drugs R&D",
        range_citation: "pubmed:41851591 — Yan 2026, Drugs R&D",
        params_note: "t½ 5.8±0.7 h fasted / 5.9±1.0 h fed; range = measured group mean±SD spread (5.1–6.9 h), consistent with cited 5–6 h consensus; Tmax ≈0.5 h fasted (1.7 h fed), oral F ≈100%, >98% renal excretion unchanged (pubmed:41851591 — Yan 2026, Drugs R&D)",
    },
    Parameterized {
        name: "Ashwagandha",
        half_life_hours: Some(9.76),
        half_life_range: Some((1.88, 9.76)),
        citation: "pubmed:38144272 — Kim 2023, Heliyon",
        range_citation: "pubmed:38144272 — Kim 2023, Heliyon",
        params_note: "Tmax 1.5–2.3 h; MRT ~14.5 h (WS-35) (pubmed:38144272 — Kim 2023, Heliyon)",
    },
    Parameterized {
        name: "Reishi Mushroom",
        half_life_hours: Some(0.66),
        half_life_range: None,
        citation: "pubmed:27747224 — Tawasri 2016, Biomed Res Int",
        range_citation: "",
        params_note: "Cmax 2.24±1.31 ng/mL; t½ 0.66±0.33 h; ascorbic acid coadministration raised ganoderic acid A exposure (pubmed:27747224 — Tawasri 2016, Biomed Res Int)",
    },
    Parameterized {
        name: "Curcumin",
        half_life_hours: Some(24.92),
        half_life_range: Some((6.74, 24.92)),
        citation: "pubmed:42225688 — Kim et al. 2026, Sci Rep",
        range_citation: "pubmed:11712783 — Cheng et al. 2001, Anticancer Res (serum peak 1-2 h, decline within 12 h after dosing up to 8 g/day); pubmed:9619120 — Shoba et al. 1998, Planta Med (piperine raises curcumin bioavailability 2000%; shortens apparent half-life)",
        params_note: "Terminal t1/2 24.92 h for free curcumin vs 6.74 h for a nano-liposomal formulation; the study itself cautions the long free-curcumin t1/2 is apparent/absorption-linked (pubmed:42225688 — Kim et al. 2026, Sci Rep) — measured after LIPOSOMAL dosing, so the tail is flip kinetics (depot release), not elimination [see issue #184]. Plain curcumin is often undetectable in serum at <=2 g (pubmed:9619120 — Shoba et al. 1998, Planta Med) and a 12 g plain-dose study found no measurable plasma levels at all (PMC3925623 — Klickovic et al.); do NOT mine tmax from that paper, its 5-6 h peaks are HO-1 mRNA expression, not PK.",
    },
    Parameterized {
        name: "Quercetin",
        half_life_hours: Some(11.0),
        half_life_range: None,
        citation: "pubmed:11361045 — Graefe et al. 2001, J Clin Pharmacol",
        range_citation: "",
        params_note: "Terminal elimination half-life ~11 h for all oral treatments (quercetin measured as plasma glucuronides); Tmax 0.7 h for quercetin-4'-glucoside/onion vs 4.3-7.0 h for rutin/buckwheat — sugar moiety controls rate (pubmed:11361045 — Graefe et al. 2001, J Clin Pharmacol). IV t1/2 only 0.7-2.4 h (pubmed:10363620 — Graefe et al. 1999, Int J Clin Pharmacol Ther).",
    },
    Parameterized {
        name: "Apigenin",
        half_life_hours: Some(2.52),
        half_life_range: Some((1.8, 4.2)),
        citation: "pubmed:34084146 — DeRango-Adem & Blay 2021, Front Pharmacol",
        range_citation: "pubmed:34084146 — DeRango-Adem & Blay 2021, Front Pharmacol",
        params_note: "Cited oral T1/2 average 2.52 +/- 0.56 h, reported range 1.8-4.2 h, Tmax 0.5-2.5 h, oral bioavailability ~30%; note the half-life estimates in that review derive from rodent PK studies (Wan 2007; Chen 2003) — human half-life not directly established (pubmed:34084146 — DeRango-Adem & Blay 2021, Front Pharmacol).",
    },
    Parameterized {
        name: "Resveratrol",
        half_life_hours: Some(9.2),
        half_life_range: None,
        citation: "pubmed:15333514 — Walle et al. 2004, Drug Metab Dispos",
        range_citation: "",
        params_note: "Plasma half-life 9.2 +/- 0.6 h refers to total radioactivity (resveratrol + metabolites) after a 25-mg oral 14C dose; parent resveratrol stayed <5 ng/mL; absorption >=70% with peak total levels 491 +/- 90 ng/mL (pubmed:15333514 — Walle et al. 2004, Drug Metab Dispos).",
    },
    Parameterized {
        name: "Berberine",
        half_life_hours: Some(5.0),
        half_life_range: Some((5.0, 5.77)),
        citation: "pubmed:35419311 — Park 2022, Transl Clin Pharmacol",
        range_citation: "pubmed:35419311 — Park 2022, Transl Clin Pharmacol",
        params_note: "Tmax ~1.0–1.3 h; CL/F 1069–1458 L/h (pubmed:35419311 — Park 2022, Transl Clin Pharmacol)",
    },
    Parameterized {
        name: "NAC (N-Acetyl Cysteine)",
        half_life_hours: Some(6.25),
        half_life_range: Some((2.27, 6.25)),
        citation: "pubmed:2029805 — Holdiness 1991, Clin Pharmacokinet",
        range_citation: "pubmed:3803419 — Borgström 1986, Eur J Clin Pharmacol; label: FDA prescribing information — Acetylcysteine injection",
        params_note: "IV terminal t½ 5.6 h, CL 0.11 L/h/kg, ~30% renal (label: FDA prescribing information — Acetylcysteine injection); oral late-phase t½ 2.27 h (pubmed:3803419 — Borgström 1986, Eur J Clin Pharmacol)",
    },
    Parameterized {
        name: "CoQ10",
        half_life_hours: Some(33.2),
        half_life_range: None,
        citation: "pubmed:3781673 — Tomono et al. 1986, Int J Clin Pharmacol Ther Toxicol",
        range_citation: "",
        params_note: "Terminal half-life 33.19 +/- 5.32 h after 100 mg d5-CoQ10 (n=16), Tmax 6.5 h, second plasma peak at 24 h from liver->VLDL redistribution (pubmed:3781673 — Tomono et al. 1986, Int J Clin Pharmacol Ther Toxicol); plasma reference interval 0.40-1.91 umol/L and tissue uptake into heart/brain mitochondria with high dosing (pubmed:16551570 — Bhagavan & Chopra 2006, Free Radic Res).",
    },
    Parameterized {
        name: "Creatine Monohydrate",
        half_life_hours: Some(1109.0),
        half_life_range: Some((832.0, 1664.0)),
        citation: "pubmed:28615996 — Kreider 2017, J Int Soc Sports Nutr",
        range_citation: "pubmed:41809290 — Aggarwal 2026, Cureus",
        params_note: "Storage ceiling ~160 mmol/kg dry muscle; 3-5 g/day maintains saturation; 4-6 weeks for elevated stores to return to baseline (all pubmed:28615996 — Kreider 2017, J Int Soc Sports Nutr); oral tmax ~2-3 h (pubmed:41809290 — Aggarwal 2026, Cureus).",
    },
    Parameterized {
        name: "Melatonin",
        half_life_hours: Some(0.75),
        half_life_range: Some((0.47, 2.1)),
        citation: "pubmed:26008214 — Harpsoe et al. 2015, European Journal of Clinical Pharmacology",
        range_citation: "label: EMA Circadin (prolonged-release melatonin 2 mg) SmPC",
        params_note: "oral Tmax 50 min (range 15-210 min); T1/2 45 min after oral and iv; oral bioavailability 15% (9-33%) (pubmed:26008214 — Harpsoe et al. 2015, Eur J Clin Pharmacol); label: complete absorption, first-pass ~85%, F ~15%, Tmax 0.75 h fasted (label: EMA Circadin SmPC)",
    },
    Parameterized {
        name: "Caffeine",
        half_life_hours: Some(5.0),
        half_life_range: Some((1.5, 9.5)),
        citation: "pubmed:27100333 — White et al. 2016, Clinical Toxicology",
        range_citation: "pubmed:6832208 — Blanchard & Sawers 1983, European Journal of Clinical Pharmacology; pubmed:10049999 — Fredholm et al. 1999, Pharmacological Reviews",
        params_note: "Tmax 29.8 +/- 8.1 min; absolute oral bioavailability ~108% (AUC oral/IV ratio); individual elimination t1/2 2.7-9.9 h in 10 healthy subjects (pubmed:6832208 — Blanchard & Sawers 1983, Eur J Clin Pharmacol); 4-6 h is the 'generally reported' mean range with adult extremes 1.5-9.5 h (pubmed:27100333)",
    },
    Parameterized {
        name: "CBD",
        half_life_hours: Some(56.0),
        half_life_range: Some((56.0, 61.0)),
        citation: "label: FDA prescribing information — EPIDIOLEX (cannabidiol) oral solution",
        range_citation: "pubmed:12648025 — Grotenhermen 2003, Clin Pharmacokinet",
        params_note: "Tmax 2.5–5 h at steady state; high-fat meal increases Cmax ~5-fold / AUC ~4-fold; sublinear dose-proportional rise in exposure (label: FDA prescribing information — EPIDIOLEX)",
    },
    Parameterized {
        name: "Kratom",
        half_life_hours: Some(23.2),
        half_life_range: Some((23.2, 67.9)),
        citation: "pubmed:25995615 — Trakulsrichai 2015, Drug Des Devel Ther",
        range_citation: "pubmed:38474495 — Huestis 2024, Molecules; pubmed:42266029 — 2026, J Anal Toxicol",
        params_note: "Tmax ~0.8–1.7 h; range = single-dose terminal 23.2 h to multiple-dose mean 67.9 h (pubmed:38474495 — Huestis 2024, Molecules); nonlinear/saturation caveat (pubmed:42266029 — 2026, J Anal Toxicol)",
    },
    Parameterized {
        name: "Memantine",
        half_life_hours: Some(70.0),
        half_life_range: Some((60.0, 80.0)),
        citation: "label: FDA prescribing information — Memantine Hydrochloride Tablets (current generic PI, 2026)",
        range_citation: "label: FDA prescribing information — Memantine Hydrochloride Tablets (current generic PI, 2026)",
        params_note: "Linear kinetics over therapeutic range; ~48% renal excretion unchanged; terminal t½ increases +95% in severe renal impairment (label: FDA prescribing information — Memantine Hydrochloride Tablets)",
    },
    Parameterized {
        name: "Estradiol Enanthate",
        half_life_hours: Some(133.7),
        half_life_range: None,
        citation: "pubmed:3814225 — Wiemeyer 1986, Arzneimittelforschung",
        range_citation: "",
        params_note: "Depot absorption t½ ≈0.46 d; serum estradiol measured elevated on days 1–31 post-injection (pubmed:3814225 — Wiemeyer 1986, Arzneimittelforschung)",
    },
];

/// Look up researched parameters for a catalog item name.
pub fn lookup(name: &str) -> Option<&'static Parameterized> {
    SEED_PK_PARAMETERS.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::seed_catalog;
    use crate::pk_assignments;

    #[test]
    fn no_duplicate_names() {
        let mut seen = std::collections::HashSet::new();
        for p in SEED_PK_PARAMETERS {
            assert!(
                seen.insert(p.name),
                "duplicate parameter row for '{name}'",
                name = p.name
            );
        }
    }

    #[test]
    fn every_parameter_row_is_cited() {
        for p in SEED_PK_PARAMETERS {
            if p.half_life_hours.is_some() {
                assert!(
                    !p.citation.trim().is_empty(),
                    "parameter row '{name}' has a point half-life but no citation (FR-036)",
                    name = p.name
                );
            }
            if p.half_life_range.is_some() {
                assert!(
                    !p.range_citation.trim().is_empty(),
                    "parameter row '{name}' has a range but no range citation (FR-036)",
                    name = p.name
                );
            }
            if let Some((lo, hi)) = p.half_life_range {
                assert!(lo > 0.0 && lo <= hi, "bad range for '{}'", p.name);
            }
            if let Some(t) = p.half_life_hours {
                assert!(t > 0.0, "non-positive half-life for '{}'", p.name);
            }
        }
    }

    /// T251–T255 checkpoint: every catalog item is EITHER parameterised
    /// here with citations OR an explicit `none` with a rationale. A model
    /// class with no citable numbers is a hole, not a model.
    #[test]
    fn every_item_parameterised_or_explicitly_none() {
        let catalog = seed_catalog();
        let mut unparameterised: Vec<String> = Vec::new();
        for item in &catalog {
            let (class, rationale) = pk_assignments::assignment_for_name(&item.name)
                .unwrap_or_else(|| panic!("'{}' has no ModelClass assignment (T205)", item.name));
            if class == crate::models::ModelClass::None {
                assert!(
                    !rationale.is_empty(),
                    "'{}' is none WITHOUT a rationale (T255 requires one)",
                    item.name
                );
                continue;
            }
            match lookup(&item.name) {
                Some(p) => assert!(
                    p.half_life_hours.is_some() || p.half_life_range.is_some(),
                    "'{}' is modelled ({class:?}) but its parameter row carries no numbers",
                    item.name
                ),
                None => unparameterised.push(format!("{} ({class:?})", item.name)),
            }
        }
        assert!(
            unparameterised.is_empty(),
            "modelled items missing researched parameters (Phase 6):\n{}",
            unparameterised.join("\n")
        );
    }
}

//! Model-class assignments for every seed-catalog substance (spec 003 T205).
//!
//! Same single-source-of-truth pattern as `SEED_TAGS`: a canonical table
//! keyed by exact catalog display name, derived into `SubstanceModel` rows.
//! Every item must appear — the coverage test fails on drift in either
//! direction (missing item, stale key).
//!
//! Assignment rule of thumb:
//! - `decay` — water-soluble / linear first-order kinetics: the single
//!   half-life describes the whole curve.
//! - `accumulation` — lipophilic or tissue-loading: repeated dosing rises
//!   toward steady state over days–weeks; a single-dose decay curve
//!   understates persistence.
//! - `plateau` — saturable pool with a ceiling (transporter or tissue
//!   saturation): more dose ≠ more active once the pool is full.
//! - `none` — no defensible one-compartment curve (live organisms,
//!   zero-order elimination, unknown parameters). `none` is a documented
//!   verdict here, never a shrug.
//!
//! Rationale is REQUIRED for every non-`decay` row (T205) — a class you
//! can't justify is a class you shouldn't assign.

use crate::models::ModelClass;
use crate::pk_params::SubstanceModel;

/// (exact catalog name, class, rationale — `""` allowed only for `decay`)
/// Researched PK integration (spec 003 Phase 6, pk_research/*.json 2026-11-17):
/// classes and rationales derive from the cited parameter tables; substances
/// whose verdict was `none` carry the documented non-verdict rationale.
const SEED_MODEL_CLASSES: &[(&str, ModelClass, &str)] = &[
    (
        "Vitamin D3",
        ModelClass::Accumulation,
        "D3 acts through the accumulating 25-hydroxylated pool: large distribution volume with DBP sequestration means body status, not a plasma decay curve, tracks effect; the 15-day 25(OH)D3 half-life reflects pool turnover over weeks.",
    ),
    (
        "Vitamin K2",
        ModelClass::Accumulation,
        "MK-7 is the canonical accumulating vitamin K: multi-day plasma retention plus documented 7-8-fold serum accumulation on chronic dosing; effect (osteocalcin/MGP carboxylation) is sustained over days rather than tracking a decay limb.",
    ),
    ("B-Complex", ModelClass::Decay, ""),
    (
        "Magnesium Glycinate",
        ModelClass::Plateau,
        "Serum Mg is homeostatically clamped, not first-order: regulated intestinal absorption (TRPM6/7), renal reabsorption with a tubular threshold, and bone buffering hold blood Mg in a narrow flat band across doses (pubmed:25540137 — de Baaij 2015, Physiol Rev). After single-dose oral Mg salts plasma shows a blunted, prolonged elevation without a clean terminal log-linear phase over the observation window (pubmed:11550076 — Ranade 2001, Am J Ther). Modeled as a plateau; the reported half-life is an elimination-side proxy from the only rigorous popPK available (IV MgSO4, one-compartment, CL 1.38 L/h, V 13.3 L, t1/2 = ln2*V/CL = 6.7 h; pubmed:32642964 — da Costa 2020, Drugs R D).",
    ),
    (
        "Zinc",
        ModelClass::Accumulation,
        "Zinc does not follow a single-compartment plasma decay: absorption (~20 +/- 5% of a meal dose) and whole-body retention are homeostatically regulated and tracer kinetics require at least two compartments (pubmed:10612077 — Watson 1999, J Trace Elem Med Biol). The dosing-relevant curve is slow tissue loading to steady state over months, whose rate constant is the whole-body 65Zn turnover half-life: 187 +/- 75 days = 4488 h in healthy controls (pubmed:3565500 — Muirhead 1986, Am J Nephrol). Hours-scale plasma falls reflect redistribution between compartments, not elimination.",
    ),
    ("Boron", ModelClass::Decay, ""),
    (
        "Omega-3 Fish Oil",
        ModelClass::Accumulation,
        "Effect (membrane enrichment / omega-3 index) tracks tissue incorporation and phospholipid turnover over days-to-weeks, not the ~2-day plasma decay limb; plasma clearance is fast relative to the slow loading of cell membranes, so model as accumulation with plasma half-lives only as the distribution limb.",
    ),
    (
        "L-Theanine",
        ModelClass::OralAbsorption,
        "Rapid absorption to a clear peak (Tmax 0.8 h after 100 mg; 45 min after 300 mg) followed by first-order terminal decline; two human studies report t1/2 1.2 h and 1.8 h, so the absorption phase must be modeled explicitly.",
    ),
    (
        "Lion's Mane",
        ModelClass::None,
        "No published human pharmacokinetics (absorption, distribution, or elimination) for the hericenones/erinacines or for any Hericium erinaceus extract; human evidence consists of small clinical-endpoint trials while disposition data remain preclinical (pubmed:40959699 — Menon 2025, Front Nutr). There is no curve to model.",
    ),
    (
        "Phosphatidylserine",
        ModelClass::None,
        "Documented non-verdict for a one-compartment curve: dietary/supplemental PS is hydrolyzed in the gut to lysophospholipid/base before absorption and reassembled into host phospholipid pools; there is no established, citable human plasma elimination half-life for oral PS, and circulating PS effect is structural/membrane-based rather than a plasma concentration curve. Parameterizing 'PS plasma t1/2' would be invented.",
    ),
    (
        "L-Tyrosine",
        ModelClass::None,
        "Endogenous amino acid; PubMed/PMC/ClinicalTrials.gov searches on 2026-11-17 (terms: 'tyrosine pharmacokinetics oral plasma humans', 'plasma tyrosine AND kinetics', author-scoped Lieberman/Nedeau queries) found no human study deriving an elimination half-life or one-compartment fit after a pharmacological oral dose. Plasma disappearance after an oral load reflects hepatic transamination, protein-pool incorporation and saturable renal handling, not one clean exponential: the human amino-acid infusion study derives per-amino-acid clearances (2.5-28 mL/s) rather than a reproducible single decay constant (pubmed:6809468 — Vilstrup et al. 1982, Eur J Clin Invest), and loading studies use downstream metabolites as absorption proxies with no tyrosine kinetics (pubmed:27577098 — Jontez et al. 2016, J Inherited Metabolic Disease). No defensible one-compartment curve; class none.",
    ),
    ("Piracetam", ModelClass::Decay, ""),
    (
        "Ashwagandha",
        ModelClass::OralAbsorption,
        "Healthy-human crossover PK of total withanolides: Tmax 1.5–2.3 h with distinct absorption and elimination phases. Half-life is strongly extract-dependent (t½ 9.756±0.517 h for WS-35 vs 1.882±0.03 h for WS-2.5, same study); point value uses WS-35 (high-glycoside, better-exposed formulation), range spans both extracts.",
    ),
    (
        "Rhodiola Rosea",
        ModelClass::None,
        "No elimination half-life for salidroside or rosavins has been published in indexed human literature; PubMed-indexed human-side data are preclinical (rat) disposition or a first-in-human study of a synthetic salidroside prodrug candidate (SHPL-49) whose elimination parameters do not describe salidroside from standardized Rhodiola extract (pubmed:39033613 — Li 2024, J Pharm Biomed Anal). No defensible one-compartment curve for the supplement.",
    ),
    (
        "Cordyceps",
        ModelClass::None,
        "Cordycepin (3'-deoxyadenosine) is destroyed by circulating adenosine deaminase, giving poor bioavailability and no usable oral human PK profile; its clinical development was only revived as the metabolically stabilized proTide prodrug NUC-7738 (pubmed:34497073 — Schwenzer 2021, Clin Cancer Res). No citable human half-life exists for cordycepin itself, and hot-water/extract preparations add unquantified mixtures.",
    ),
    (
        "Reishi Mushroom",
        ModelClass::OralAbsorption,
        "The only indexed human PK study: Ling Zhi (Reishi) preparation in healthy adults (n=12) with ganoderic acid A as marker — rapid absorption (Tmax 0.54±0.10 h) and very short elimination (t½ 0.66±0.33 h), ng/mL-scale exposure. Range left null: single study, only within-group SD reported. Extract standardization varies, so treat as marker-compound kinetics.",
    ),
    (
        "Umcka (Pelargonium)",
        ModelClass::None,
        "No human PK parameter set is established for EPs 7630 (Umckaloabo): the active constituents are not fully identified and plasma levels of extract markers are not measurable at therapeutic doses. The only indexed human quantitation is of marker compounds given in an ultra-low-exposure combination product, where epicatechin (the PSE marker) showed erratic trace kinetics (t½ 2.10 h fasting / 9.33 h fed, ng/L Cmax) that do not characterize the extract (pubmed:35419311 — Park 2022, Transl Clin Pharmacol).",
    ),
    (
        "Curcumin",
        ModelClass::None,
        "Refused on both sides of the curve. Absorption: no unformulated primary can exist — plain oral curcumin stays at/below the limit of quantitation in plasma (low-ng/mL detections in pubmed:11712783 Cheng 2001 are at assay floor; a 12 g plain-dose study found none at all, PMC3925623), so every tmax/t1/2 in the literature belongs to an enhancer formulation (nanoparticle THERACURMIN t½ 9.7 h pubmed:21603867; IV liposomal undetectable post-infusion pubmed:30074076). Elimination: the seed 24.92 h tail (pubmed:42225688 — Kim 2026, Sci Rep) is explicitly flagged by its own source as apparent/absorption-linked flip kinetics (depot-like release), not elimination — #184. A one-compartment curve would be built entirely from artifacts; a formulation-named model ('curcumin (nanoparticle)') is the only honest parameterization, pending a modeling decision. [see issues #184/#185]",
    ),
    (
        "Quercetin",
        ModelClass::None,
        "Model-class failure, refused per #185 decision (no speculative enterohepatic class for one substance). Plasma contains virtually only conjugated metabolites, and the oral terminal half-life (~11 h) far exceeds the IV half-life (0.7-2.4 h) — classic absorption/conjugation-controlled (flip-flop-prone) kinetics. Verified primary absorption data exists (pubmed:11151743 — Erlund 2000, n=16 crossover, aglycone Tmax 1.9/2.7/4.8 h at 8/20/50 mg) but no Bateman entry is derivable: at the highest aglycone dose ka/ke collapses to 1.11 (pubmed:18241083 — Moon 2008 terminal t1/2 3.5 h), and plasma curves show enterohepatic re-entry peaks a one-compartment kernel cannot represent. The measurement records stay in pk_parameters as disclaimed data.",
    ),
    (
        "Apigenin",
        ModelClass::None,
        "Metabolite-driven failure, refused per #185 decision. Human primary ADME exists but the parent compound is not what circulates: apigenin per se is poorly absorbed (urinary metabolite recovery 0.5%, pubmed:35452808 — Borges 2022, Free Radic Biol Med, NCT03526081), and plasma exposure is apigenin-4'-glucuronide with Cmax time set by the matrix, not an absorption rate constant (2 h chamomile tea upper-GIT; 4 h parsley drink and 6 h parsley+yogurt lower-GIT/colonic release of the apiosylglucoside). No plasma half-life reported — elimination was assessed by urinary excretion. Any ka mined from those Cmax times would encode the food vehicle.",
    ),
    (
        "Resveratrol",
        ModelClass::None,
        "Metabolite-driven failure, refused per #185 decision (no speculative metabolite class for one substance). >=70% absorbed but almost entirely sulfate/glucuronide-hydrogenated metabolites in plasma — the measured 9.2 h limb is metabolite disposition after very rapid intestinal first-pass, so a plain one-compartment decay on parent drug is invalid. Parent plasma levels are low-ng/mL; a Bateman ka would fit the conjugate and call it absorption. The measurement records stay in pk_parameters as disclaimed data.",
    ),
    (
        "Berberine",
        ModelClass::OralAbsorption,
        "Indexed human oral PK with clear absorption phase (Tmax ~1–2 h) and terminal elimination t½ 5.00±2.17 h (fed) / 5.77 h (fasting); apparent CL/F ~1070–1460 L/h and ng/L–low-µg/L exposures reflect extensive first-pass metabolism and very low systemic availability. Caveat: values come from a berberine-containing combination product (UI026) in healthy subjects — the only citable human parameter set found.",
    ),
    (
        "NAC (N-Acetyl Cysteine)",
        ModelClass::OralAbsorption,
        "Oral absorption with extensive first-pass metabolism (intact NAC systemic availability low; decline poly-exponential), so the full curve needs absorption + elimination phases. Reported human terminal half-lives vary by study/phase: oral terminal t½ 6.25 h; a separate study reports late-elimination phase 2.27 h; IV terminal 5.6 h. Range = 2.27–6.25 h.",
    ),
    (
        "CoQ10",
        ModelClass::Accumulation,
        "Lipoprotein/liver handling and lipophilic tissue loading (biphasic curve, 24-h second peak) mean plasma level is a redistribution proxy while mitochondrial pools load over weeks of dosing — tissue-loading model with steady state over days-weeks, plasma t1/2 noted in params.",
    ),
    (
        "Creatine Monohydrate",
        ModelClass::Plateau,
        "Muscle creatine is a saturable pool with a hard ceiling of about 160 mmol/kg dry muscle mass that a normal diet fills only to 60-80%, so repeated dosing drives the pool to the ceiling and no further; the dose-response tracks pool enrichment, not plasma concentration (pubmed:28615996 — Kreider 2017, J Int Soc Sports Nutr). Plasma creatine is cleared rapidly into muscle and urine, so a plasma half-life does not describe the effect time course; the value given is whole-pool turnover: 1-2% of intramuscular creatine degrades to creatinine and is excreted per day, t1/2 = ln2/(0.01-0.02/day) = 34.7-69.3 days, midpoint ~46.2 days = 1109 h (pubmed:28615996 — Kreider 2017, J Int Soc Sports Nutr).",
    ),
    (
        "Collagen Peptides",
        ModelClass::None,
        "Collagen peptides are a digestible mixture of gelatin-derived peptides, not a single drug-like analyte. Only specific food-derived di-/tri-peptides (Pro-Hyp, Gly-Pro-Hyp) transiently appear in plasma and are excreted unchanged in urine - a transient absorbed peptide flux rather than a persistent compartment (pubmed:26934933 — Yamamoto 2016, Biol Pharm Bull; pubmed:16076145 — Iwai 2005, J Agric Food Chem). The bulk of the dose is hydrolyzed to ordinary amino acids whose kinetics belong to the general amino-acid pool, so no defensible one-compartment curve exists for collagen peptides as such.",
    ),
    (
        "Probiotic",
        ModelClass::None,
        "A live organism, not a molecule: probiotic persistence is decided by the host's baseline microbiome and immune status (not a rate constant), and detection tracks transient gut passage/turnover over days rather than exponential elimination (pubmed:30193112 — Zmora 2018, Cell). Any single half-life would be strain-, product-, host-, and matrix-dependent to the point of being meaningless; survival through gastric acid/bile and strain-specific growth add non-pharmacokinetic nonlinearities.",
    ),
    (
        "Melatonin",
        ModelClass::OralAbsorption,
        "~85% first-pass metabolism makes oral bioavailability only ~15% (9-33%), so post-dose plasma levels are dominated by an absorption/first-pass phase (Tmax ~50 min) ahead of a short first-order elimination (T1/2 45 min; 28-126 min across studies); an oral_absorption model is required.",
    ),
    (
        "Caffeine",
        ModelClass::OralAbsorption,
        "Oral dose gives a distinct absorption peak (Tmax ~30 min, Cmax ~19 mg/L after 5 mg/kg) before first-order CYP1A2-mediated elimination, so absorption + elimination phases are both needed; single-half-life decay alone would misplace the early curve.",
    ),
    (
        "Cyclazodone",
        ModelClass::None,
        "No human PK evidence exists at any level: PubMed (eutils, 2026-11-17) returns 0 records for 'cyclazodine' with wildcard (no title/abstract mention anywhere in MEDLINE), ClinicalTrials.gov totalCount=0, and PubChem PUG REST has no compound record for the name (HTTP 404). With no bioanalytical or clinical study indexed, no half-life or compartment model can be constructed; honest verdict is none.",
    ),
    (
        "N-Methyl-Cyclazodone",
        ModelClass::None,
        "Worse-evidenced N-methyl analogue of cyclazodone: PubMed wildcard search returns 0 records (2026-11-17) and ClinicalTrials.gov totalCount=0. No PK reported in any species, no human dosing, no indexed preclinical literature from which an allometric estimate could even be derived; class none.",
    ),
    (
        "CBD",
        ModelClass::Accumulation,
        "Highly lipophilic with enormous apparent volume of distribution (20,963–42,849 L) and >94% protein binding; plasma concentrations accumulate with repeated dosing and label-reported plasma half-life is 56–61 h after twice-daily dosing for 7 days — tissue loading to steady state over days, not a same-day decay curve. Single-dose half-lives are shorter, but the accumulation profile governs any dosing model.",
    ),
    (
        "Alcohol",
        ModelClass::None,
        "Ethanol elimination is capacity-limited, not first-order: ADH is saturated across the social-to-intoxication concentration range, so blood levels fall at an essentially constant amount per time (zero-order / Michaelis-Menten) and an 'elimination half-life' is mathematically undefined and grows as concentration declines (pubmed:12489977 — Norberg 2003, Clin Pharmacokinet; pubmed:20304569 — Jones 2010, Forensic Sci Int). The literature recommends multicompartment Michaelis-Menten models, not a half-life curve.",
    ),
    (
        "Kratom",
        ModelClass::Accumulation,
        "Mitragynine (the dominant alkaloid) shows a deep second compartment: first human PK (n=9) gives terminal t½ 23.24±16.07 h with apparent Vd 38.04±24.32 L/kg (two-compartment oral model). Repeated-dose work reports mean half-lives rising to 43.4 h (single) and 67.9 h (multiple doses), with possible enzyme saturation at higher doses — tissue-loading behavior reaching steady state over days of daily use, so modeled as accumulation.",
    ),
    (
        "9-Me-BC",
        ModelClass::None,
        "All indexed literature is preclinical: mesencephalic cell-culture and rodent dopaminergic neuroprotection/differentiation work (pubmed:21651332 — Polanski et al. 2011, Expert Rev Neurother) and astrocyte MAO-inhibition/MPP+ models (pubmed:32285253 — Keller et al. 2020, J Neural Transm); PubMed queries '9-methyl-beta-carboline' and '9-Me-BC' (2026-11-17) return no human study, and ClinicalTrials.gov totalCount=0. No human PK studies found and no published animal PK disposition data to extrapolate from; class none.",
    ),
    ("Memantine", ModelClass::Decay, ""),
    (
        "Estradiol Enanthate",
        ModelClass::ImDepotAbsorption,
        "IM depot ester in menopausal women after 10 mg i.m. (pubmed:3814225 — Wiemeyer 1986, RIA days 1-31): absorption rate constant Kabs = 1.5050 d⁻¹ → depot t½ 0.46 d, elimination rate constant K = 0.12445 d⁻¹ → t½(elim) = 5.57 d = 133.7 h; serum estradiol stayed elevated through ~day 31. Kabs/K = 12.1, so the terminal slope is elimination-limited, NOT flip-flop (earlier label corrected). Oral bioavailability is irrelevant — the recorded profile is the injection. Range null: single primary study.",
    ),
];

/// Assignment lookup by exact catalog display name.
pub fn assignment_for_name(name: &str) -> Option<(ModelClass, &'static str)> {
    SEED_MODEL_CLASSES
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, c, r)| (*c, *r))
}

/// Build the full `SubstanceModel` set for a catalog: class + numeric
/// half-life (where parsed) + mandatory provenance + rationale.
/// Panics in debug builds on an unassigned name; in release an unassigned
/// item becomes an explicit `none` row whose rationale says so — visible,
/// never silently absent.
pub fn build_models(catalog: &[crate::models::CatalogItem]) -> Vec<SubstanceModel> {
    catalog
        .iter()
        .map(|item| {
            let (class, rationale) = assignment_for_name(&item.name)
                .unwrap_or_else(|| {
                    debug_assert!(false, "catalog item '{name}' has no ModelClass assignment (T205 coverage test should have caught this)", name = item.name);
                    (ModelClass::None, "UNASSIGNED: catalog item not present in SEED_MODEL_CLASSES")
                });
            let provenance = match item.half_life.as_deref() {
                Some(raw) => format!("catalog seed string '{raw}' + T205 assignment table"),
                None => "catalog item has no half_life string; T205 assignment only".to_string(),
            };
            // Phase 6 (T251/T256): researched parameters with per-value
            // citations supersede the seed-string parse. The seed string
            // stays chained in the provenance so the audit trail shows both
            // the original catalog origin and the researched source.
            let researched = crate::pk_parameters::lookup(&item.name);
            let (half_life_hours, half_life_range, provenance, params) = match researched {
                Some(p) => {
                    let mut prov = match (p.half_life_hours, p.half_life_range) {
                        (Some(_), _) if !p.citation.is_empty() => p.citation.to_string(),
                        _ => String::new(),
                    };
                    if !p.range_citation.is_empty() {
                        if !prov.is_empty() {
                            prov.push_str(" | ");
                        }
                        prov.push_str(p.range_citation);
                    }
                    if !p.params_note.is_empty() {
                        if !prov.is_empty() {
                            prov.push_str(" | ");
                        }
                        prov.push_str("params: ");
                        prov.push_str(p.params_note);
                    }
                    let params = if p.params_note.is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::json!({ "notes": p.params_note })
                    };
                    let provenance = if prov.is_empty() {
                        provenance
                    } else {
                        format!("{prov} (supersedes: {provenance})")
                    };
                    (
                        p.half_life_hours.or(item.half_life_hours),
                        p.half_life_range.or(item.half_life_range),
                        provenance,
                        params,
                    )
                }
                None => (
                    item.half_life_hours,
                    item.half_life_range,
                    provenance,
                    serde_json::Value::Null,
                ),
            };
            // #182: OralAbsorption models need a resorption parameter to
            // simulate. SEED table is the single producer; substances not
            // in it keep refusing to simulate (honest FR-036 refusal,
            // never a guessed rate).
            let mut provenance = provenance;
            let mut params = params;
            if let Some(abs) = crate::pk_absorption::lookup(&item.name)
                .filter(|_| class == ModelClass::OralAbsorption)
            {
                if !params.is_object() {
                    params = serde_json::Value::Object(serde_json::Map::new());
                }
                params["absorption_half_life_hours"] = serde_json::json!(abs.absorption_half_life_hours);
                provenance.push_str(" | absorption: ");
                provenance.push_str(abs.provenance);
            }
            // #183: the depot twin. Oral rates must never be injected
            // here; DEPOT_RESEARCH is measured-after-injection only.
            if let Some(dep) = crate::pk_depot::lookup(&item.name)
                .filter(|_| class == ModelClass::ImDepotAbsorption)
            {
                if !params.is_object() {
                    params = serde_json::Value::Object(serde_json::Map::new());
                }
                params["absorption_half_life_hours"] = serde_json::json!(dep.absorption_half_life_hours);
                provenance.push_str(" | depot release: ");
                provenance.push_str(dep.provenance);
            }
            SubstanceModel {
                substance: item.name.clone(),
                model_class: class,
                half_life_hours,
                half_life_range,
                params,
                provenance,
                catalog_version: item.version,
                rationale: (!rationale.is_empty()).then(|| rationale.to_string()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::seed_catalog;

    #[test]
    fn every_catalog_item_has_exactly_one_assignment() {
        let catalog = seed_catalog();
        let table_names: std::collections::HashSet<&str> =
            SEED_MODEL_CLASSES.iter().map(|(n, ..)| *n).collect();
        assert_eq!(
            table_names.len(),
            SEED_MODEL_CLASSES.len(),
            "duplicate key in SEED_MODEL_CLASSES"
        );

        let missing: Vec<&str> = catalog
            .iter()
            .filter(|i| !table_names.contains(i.name.as_str()))
            .map(|i| i.name.as_str())
            .collect();
        assert!(
            missing.is_empty(),
            "catalog items with no ModelClass assignment: {missing:?}"
        );

        // Stale table keys (name changed in catalog) are drift too.
        let catalog_names: std::collections::HashSet<&str> =
            catalog.iter().map(|i| i.name.as_str()).collect();
        let stale: Vec<&&str> = table_names
            .iter()
            .filter(|n| !catalog_names.contains(**n))
            .collect();
        assert!(
            stale.is_empty(),
            "SEED_MODEL_CLASSES keys not in catalog: {stale:?}"
        );
    }

    #[test]
    fn non_decay_assignments_carry_rationale() {
        for (name, class, rationale) in SEED_MODEL_CLASSES {
            if *class != ModelClass::Decay {
                assert!(
                    !rationale.is_empty(),
                    "'{name}' is {class:?} but has no rationale (T205 requires one)"
                );
            }
        }
    }

    #[test]
    fn parse_failures_are_assigned_none() {
        // The T204 audit pins 9-Me-BC as the only parse failure; it MUST be
        // `none` — a class over a number that doesn't parse is a contradiction.
        let (class, rationale) = assignment_for_name("9-Me-BC").unwrap();
        assert_eq!(class, ModelClass::None);
        assert!(!rationale.is_empty());
    }

    #[test]
    fn spot_checks_match_pharmacology() {
        // Spec 003's own examples plus the judgment calls.
        assert_eq!(
            assignment_for_name("Creatine Monohydrate").unwrap().0,
            ModelClass::Plateau
        );
        assert_eq!(
            assignment_for_name("Vitamin D3").unwrap().0,
            ModelClass::Accumulation
        );
        assert_eq!(
            assignment_for_name("CBD").unwrap().0,
            ModelClass::Accumulation
        );
        assert_eq!(assignment_for_name("Alcohol").unwrap().0, ModelClass::None);
        assert_eq!(
            assignment_for_name("Probiotic").unwrap().0,
            ModelClass::None
        );
        assert_eq!(
            assignment_for_name("Caffeine").unwrap().0,
            ModelClass::OralAbsorption
        );
    }

    #[test]
    fn build_models_is_total_and_carries_parsed_numbers() {
        let models = build_models(&seed_catalog());
        assert_eq!(
            models.len(),
            seed_catalog().len(),
            "one row per item, total audit"
        );
        for m in &models {
            assert!(
                !m.provenance.is_empty(),
                "{:?} missing provenance",
                m.substance
            );
        }
        let caffeine = models.iter().find(|m| m.substance == "Caffeine").unwrap();
        assert_eq!(caffeine.half_life_hours, Some(5.0));
        assert!(caffeine.has_numeric_half_life());
        let alcohol = models.iter().find(|m| m.substance == "Alcohol").unwrap();
        assert_eq!(alcohol.model_class, ModelClass::None);
        assert_eq!(alcohol.half_life_range, Some((4.0, 6.0))); // parsed but not modelled — both facts kept
        let nine = models.iter().find(|m| m.substance == "9-Me-BC").unwrap();
        assert!(!nine.has_numeric_half_life());
    }

    #[test]
    fn curcumin_refused_not_flip_kinetics_simulated() {
        // #184: the seed 24.92 h tail is an apparent flip-kinetics artifact
        // (depot-like release, own source's caveat). Simulating it as ke
        // would encode absorption into elimination. If this test ever fails,
        // a provenance-free tail has crept back into a curve.
        let (class, rationale) = assignment_for_name("Curcumin").expect("curcumin assigned");
        assert_eq!(class, ModelClass::None);
        assert!(rationale.contains("flip kinetics"));
        assert!(rationale.contains("24.92"));
    }

    #[test]
    fn model_class_failures_refused_not_simulated() {
        // #185 decision: quercetin/resveratrol/apigenin exceed what a
        // one-compartment kernel can represent. Refusing is the model —
        // a new class needs at least two justified substances before
        // implementation, and none of these three can share a kernel.
        // If this test fails, a degenerate curve has been re-enabled.
        for (name, marker) in [
            ("Quercetin", "Model-class failure"),
            ("Resveratrol", "Metabolite-driven failure"),
            ("Apigenin", "Metabolite-driven failure"),
        ] {
            let (class, rationale) =
                assignment_for_name(name).unwrap_or_else(|| panic!("{name} assigned"));
            assert_eq!(class, ModelClass::None, "{name} must stay refused");
            assert!(
                rationale.contains(marker),
                "{name} rationale lost its refusal class"
            );
            assert!(
                rationale.contains("#185"),
                "{name} rationale lost its decision anchor"
            );
        }
    }
}

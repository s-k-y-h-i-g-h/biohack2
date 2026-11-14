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
const SEED_MODEL_CLASSES: &[(&str, ModelClass, &str)] = &[
    // ── Vitamins & minerals ──────────────────────────────────────────────
    ("Vitamin D3", ModelClass::Accumulation,
     "Fat-soluble; sequestered in adipose and re-released — serum 25(OH)D reaches steady state over ~6-8 weeks of daily dosing, so a 24h plasma figure massively understates persistence."),
    ("Vitamin K2", ModelClass::Accumulation,
     "Fat-soluble menaquinone; hepatic and extrahepatic tissue storage with slow clearance (days), steady state over weeks."),
    ("B-Complex", ModelClass::Decay, ""),
    ("Magnesium Glycinate", ModelClass::Decay, ""),
    ("Zinc", ModelClass::Decay, ""),
    ("Boron", ModelClass::Decay, ""),
    // ── Amino acids & derivatives ────────────────────────────────────────
    ("L-Theanine", ModelClass::Decay, ""),
    ("L-Tyrosine", ModelClass::Decay, ""),
    // ── Lipids & membrane actives ────────────────────────────────────────
    ("Omega-3 Fish Oil", ModelClass::Accumulation,
     "EPA/DHA incorporate into membrane phospholipids over weeks (RBC omega-3 index turnover ~120 days); the effect curve tracks tissue enrichment, not the plasma clearance of a single dose."),
    ("Phosphatidylserine", ModelClass::Decay, ""),
    ("CoQ10", ModelClass::Accumulation,
     "Plasma half-life ~24-33h but tissue/mitochondrial saturation builds over 2-4 weeks of daily dosing; single-dose decay understates the steady-state rise."),
    // ── Creatine ─────────────────────────────────────────────────────────
    ("Creatine Monohydrate", ModelClass::Plateau,
     " Muscle creatine pool saturates via the SAT1 transporter (total pool ~160 mmol/kg dry mass); above saturation, additional dose is oxidised to creatinine and excreted — more dose ≠ more active."),
    // ── Botanicals & adaptogens (linear kinetics) ────────────────────────
    ("Lion's Mane", ModelClass::Decay, ""),
    ("Ashwagandha", ModelClass::Decay, ""),
    ("Rhodiola Rosea", ModelClass::Decay, ""),
    ("Cordyceps", ModelClass::Decay, ""),
    ("Reishi Mushroom", ModelClass::Decay, ""),
    ("Umcka (Pelargonium)", ModelClass::Decay, ""),
    ("Curcumin", ModelClass::Decay, ""),
    ("Quercetin", ModelClass::Decay, ""),
    ("Apigenin", ModelClass::Decay, ""),
    ("Resveratrol", ModelClass::Decay, ""),
    ("Berberine", ModelClass::Decay, ""),
    // ── Antioxidants / thiol precursors ──────────────────────────────────
    ("NAC (N-Acetyl Cysteine)", ModelClass::Decay, ""),
    // ── Proteins ─────────────────────────────────────────────────────────
    ("Collagen Peptides", ModelClass::Decay, ""),
    // ── Live organisms ───────────────────────────────────────────────────
    ("Probiotic", ModelClass::None,
     "Not a chemical dose curve: effect comes from transient colonisation and immune signalling, not a circulating compound. A one-compartment model would be fake precision."),
    // ── Hormones & neuropeptides ─────────────────────────────────────────
    ("Melatonin", ModelClass::Decay, ""),
    ("Estradiol Enanthate", ModelClass::Decay,
     ""), // depot release-rate-limited but approximately first-order over the ~8d window; decay is the honest v1 model
    // ── Stimulants ───────────────────────────────────────────────────────
    ("Caffeine", ModelClass::Decay, ""),
    // ── Designer / research compounds ────────────────────────────────────
    ("Cyclazodone", ModelClass::Decay, ""),
    ("N-Methyl-Cyclazodone", ModelClass::Decay, ""),
    ("9-Me-BC", ModelClass::None,
     "No reliable half-life in the catalog ('unknown') and thin human PK data — refuse to model rather than invent a curve."),
    // ── Cannabinoids ─────────────────────────────────────────────────────
    ("CBD", ModelClass::Accumulation,
     "Highly lipophilic with substantial tissue redistribution; chronic dosing shows multi-day apparent half-life and rising steady-state plasma levels — single-dose decay understates persistence."),
    // ── Depressants ──────────────────────────────────────────────────────
    ("Alcohol", ModelClass::None,
     "Zero-order elimination (~7 g/h, capacity-limited ADH) — a first-order decay curve would be wrong exactly where it matters (the descending limb). Linear-elimination model is a separate Phase 2 decision, not a decay-with-parameters hack."),
    // ── Opioid-receptor botanical ────────────────────────────────────────
    ("Kratom", ModelClass::Decay, ""),
    // ── Prescription ─────────────────────────────────────────────────────
    ("Memantine", ModelClass::Decay, ""),
    // ── Nootropics ───────────────────────────────────────────────────────
    ("Piracetam", ModelClass::Decay, ""),
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
            SubstanceModel {
                substance: item.name.clone(),
                model_class: class,
                half_life_hours: item.half_life_hours,
                half_life_range: item.half_life_range,
                params: serde_json::Value::Null,
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
        assert!(missing.is_empty(), "catalog items with no ModelClass assignment: {missing:?}");

        // Stale table keys (name changed in catalog) are drift too.
        let catalog_names: std::collections::HashSet<&str> =
            catalog.iter().map(|i| i.name.as_str()).collect();
        let stale: Vec<&&str> = table_names.iter().filter(|n| !catalog_names.contains(**n)).collect();
        assert!(stale.is_empty(), "SEED_MODEL_CLASSES keys not in catalog: {stale:?}");
    }

    #[test]
    fn non_decay_assignments_carry_rationale() {
        for (name, class, rationale) in SEED_MODEL_CLASSES {
            if *class != ModelClass::Decay {
                assert!(!rationale.is_empty(), "'{name}' is {class:?} but has no rationale (T205 requires one)");
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
        assert_eq!(assignment_for_name("Creatine Monohydrate").unwrap().0, ModelClass::Plateau);
        assert_eq!(assignment_for_name("Vitamin D3").unwrap().0, ModelClass::Accumulation);
        assert_eq!(assignment_for_name("CBD").unwrap().0, ModelClass::Accumulation);
        assert_eq!(assignment_for_name("Alcohol").unwrap().0, ModelClass::None);
        assert_eq!(assignment_for_name("Probiotic").unwrap().0, ModelClass::None);
        assert_eq!(assignment_for_name("Caffeine").unwrap().0, ModelClass::Decay);
    }

    #[test]
    fn build_models_is_total_and_carries_parsed_numbers() {
        let models = build_models(&seed_catalog());
        assert_eq!(models.len(), seed_catalog().len(), "one row per item, total audit");
        for m in &models {
            assert!(!m.provenance.is_empty(), "{:?} missing provenance", m.substance);
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
}

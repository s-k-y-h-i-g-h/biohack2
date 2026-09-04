//! 27-substance seed database ported from the biohack CLI.

use crate::models::{CatalogItem, DosageRange, ItemType};
use uuid::Uuid;

/// Returns the full 27-substance seed catalog.
pub fn seed_catalog() -> Vec<CatalogItem> {
    vec![
        // Vitamins
        make_item("vit-d3", "Vitamin D3", ItemType::Supplement, Some(DosageRange { min: 1000.0, max: 5000.0, unit: "IU".to_string() }), "24h", &[], &["Consult physician if taking thiazide diuretics"]),
        make_item("vit-k2", "Vitamin K2", ItemType::Supplement, Some(DosageRange { min: 50.0, max: 200.0, unit: "mcg".to_string() }), "72h", &["blood-thinners"], &["May interact with blood thinners like warfarin"]),
        make_item("b-complex", "B-Complex", ItemType::Supplement, Some(DosageRange { min: 1.0, max: 1.0, unit: "serving".to_string() }), "6h", &[], &[]),
        // Minerals
        make_item("mg-glycinate", "Magnesium Glycinate", ItemType::Supplement, Some(DosageRange { min: 200.0, max: 400.0, unit: "mg".to_string() }), "4h", &[], &["May cause diarrhea at high doses"]),
        make_item("zinc", "Zinc", ItemType::Supplement, Some(DosageRange { min: 15.0, max: 30.0, unit: "mg".to_string() }), "24h", &[], &["Long-term high doses may cause copper deficiency"]),
        make_item("boron", "Boron", ItemType::Supplement, Some(DosageRange { min: 1.0, max: 3.0, unit: "mg".to_string() }), "24h", &[], &[]),
        // Omega / Fats
        make_item("omega-3", "Omega-3 Fish Oil", ItemType::Supplement, Some(DosageRange { min: 1000.0, max: 3000.0, unit: "mg".to_string() }), "24h", &["blood-thinners"], &["May increase bleeding risk with anticoagulants"]),
        // Nootropics / Cognitive
        make_item("l-theanine", "L-Theanine", ItemType::Supplement, Some(DosageRange { min: 100.0, max: 200.0, unit: "mg".to_string() }), "3h", &[], &[]),
        make_item("lion-mane", "Lion's Mane", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "8h", &[], &[]),
        make_item("phosphatidylserine", "Phosphatidylserine", ItemType::Supplement, Some(DosageRange { min: 100.0, max: 300.0, unit: "mg".to_string() }), "8h", &[], &[]),
        make_item("tyrosine", "L-Tyrosine", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "2h", &[], &["May interact with thyroid medication"]),
        // Adaptogens / Herbs
        make_item("ashwagandha", "Ashwagandha", ItemType::Supplement, Some(DosageRange { min: 300.0, max: 600.0, unit: "mg".to_string() }), "24h", &["thyroid-medication"], &["May affect thyroid function"]),
        make_item("rhodiola", "Rhodiola Rosea", ItemType::Supplement, Some(DosageRange { min: 200.0, max: 400.0, unit: "mg".to_string() }), "8h", &[], &["May interact with antidepressants (serotonergic)"]),
        make_item("cordyceps", "Cordyceps", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "8h", &[], &["May stimulate immune system"]),
        make_item("reishi", "Reishi Mushroom", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "8h", &["blood-thinners"], &["May interact with blood thinners and immunosuppressants"]),
        make_item("umcka", "Umcka (Pelargonium)", ItemType::Supplement, Some(DosageRange { min: 20.0, max: 30.0, unit: "mg".to_string() }), "8h", &[], &["May interact with certain medications"]),
        // Anti-inflammatory
        make_item("curcumin", "Curcumin", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "8h", &["blood-thinners"], &["May interact with blood thinners"]),
        make_item("quercetin", "Quercetin", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }), "8h", &[], &["May interact with blood thinners and antibiotics"]),
        make_item("apigenin", "Apigenin", ItemType::Supplement, Some(DosageRange { min: 25.0, max: 50.0, unit: "mg".to_string() }), "8h", &[], &["May interact with blood thinners"]),
        // Metabolic / Longevity
        make_item("resveratrol", "Resveratrol", ItemType::Supplement, Some(DosageRange { min: 100.0, max: 500.0, unit: "mg".to_string() }), "8h", &[], &[]),
        make_item("berberine", "Berberine", ItemType::Supplement, Some(DosageRange { min: 500.0, max: 1500.0, unit: "mg".to_string() }), "4h", &["metformin"], &["May interact with diabetes medications"]),
        make_item("nac", "NAC (N-Acetyl Cysteine)", ItemType::Supplement, Some(DosageRange { min: 600.0, max: 1200.0, unit: "mg".to_string() }), "6h", &[], &["May interact with nitroglycerin"]),
        make_item("coq10", "CoQ10", ItemType::Supplement, Some(DosageRange { min: 100.0, max: 200.0, unit: "mg".to_string() }), "24h", &["blood-thinners"], &["May reduce effectiveness of blood thinners"]),
        // Performance
        make_item("creatine", "Creatine Monohydrate", ItemType::Supplement, Some(DosageRange { min: 3.0, max: 5.0, unit: "g".to_string() }), "24h", &[], &["Stay hydrated"]),
        // Gut / Beauty
        make_item("collagen", "Collagen Peptides", ItemType::Supplement, Some(DosageRange { min: 10.0, max: 20.0, unit: "g".to_string() }), "6h", &[], &[]),
        make_item("probiotic", "Probiotic", ItemType::Supplement, Some(DosageRange { min: 1.0, max: 10.0, unit: "CFU billion".to_string() }), "24h", &[], &["Consult physician if immunocompromised"]),
        // Sleep
        make_item("melatonin", "Melatonin", ItemType::Supplement, Some(DosageRange { min: 0.5, max: 5.0, unit: "mg".to_string() }), "2h", &[], &["May cause drowsiness"]),
    ]
}

fn make_item(
    id: &str,
    name: &str,
    category: ItemType,
    dosage_range: Option<DosageRange>,
    half_life: &str,
    contraindications: &[&str],
    warnings: &[&str],
) -> CatalogItem {
    CatalogItem {
        id: Uuid::parse_str(id).unwrap_or(Uuid::new_v4()),
        name: name.to_string(),
        category,
        dosage_range,
        half_life: Some(half_life.to_string()),
        contraindications: contraindications.iter().map(|s| s.to_string()).collect(),
        warnings: warnings.iter().map(|s| s.to_string()).collect(),
        is_custom: false,
        source: Some("biohack CLI seed".to_string()),
        version: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_catalog_has_27_items {
        let catalog = seed_catalog();
        assert_eq!(catalog.len(), 27);
    }

    #[test]
    fn test_all_items_have_dosage_range {
        let catalog = seed_catalog();
        for item in &catalog {
            assert!(item.dosage_range.is_some(), "Item {} missing dosage_range", item.name);
            let dr = item.dosage_range.as_ref().unwrap();
            assert!(dr.min > 0.0, "Item {} has min <= 0", item.name);
            assert!(dr.max >= dr.min, "Item {} has max < min", item.name);
        }
    }

    #[test]
    fn test_known_interactions_present {
        let catalog = seed_catalog();
        let omega = catalog.iter().find(|i| i.id.to_string() == "omega-3").unwrap();
        assert!(omega.contraindications.iter().any(|c| c == "blood-thinners"));

        let vitk2 = catalog.iter().find(|i| i.id.to_string() == "vit-k2").unwrap();
        assert!(vitk2.contraindications.iter().any(|c| c == "blood-thinners"));

        let berberine = catalog.iter().find(|i| i.id.to_string() == "berberine").unwrap();
        assert!(berberine.contraindications.iter().any(|c| c == "metformin"));
    }
}

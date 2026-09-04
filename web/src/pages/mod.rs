use leptos::*;
use leptos::prelude::*;
use tachys::html::element::ElementChild;

/// Lightweight catalog for WASM frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CatalogItem {
    pub name: String,
    pub category: String,
    pub dosage_range: Option<DosageRange>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DosageRange {
    pub min: f64,
    pub max: f64,
    pub unit: String,
}

pub fn seed_catalog() -> Vec<CatalogItem> {
    vec![
        CatalogItem { name: "Vitamin D3".to_string(), category: "vitamin".to_string(), dosage_range: Some(DosageRange { min: 1000.0, max: 5000.0, unit: "IU".to_string() }) },
        CatalogItem { name: "Vitamin K2".to_string(), category: "vitamin".to_string(), dosage_range: Some(DosageRange { min: 50.0, max: 200.0, unit: "mcg".to_string() }) },
        CatalogItem { name: "B-Complex".to_string(), category: "vitamin".to_string(), dosage_range: Some(DosageRange { min: 1.0, max: 1.0, unit: "serving".to_string() }) },
        CatalogItem { name: "Magnesium Glycinate".to_string(), category: "mineral".to_string(), dosage_range: Some(DosageRange { min: 200.0, max: 400.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Zinc".to_string(), category: "mineral".to_string(), dosage_range: Some(DosageRange { min: 15.0, max: 30.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Omega-3 Fish Oil".to_string(), category: "fat".to_string(), dosage_range: Some(DosageRange { min: 1000.0, max: 3000.0, unit: "mg".to_string() }) },
        CatalogItem { name: "L-Theanine".to_string(), category: "nootropic".to_string(), dosage_range: Some(DosageRange { min: 100.0, max: 200.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Lion's Mane".to_string(), category: "nootropic".to_string(), dosage_range: Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Ashwagandha".to_string(), category: "adaptogen".to_string(), dosage_range: Some(DosageRange { min: 300.0, max: 600.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Creatine Monohydrate".to_string(), category: "performance".to_string(), dosage_range: Some(DosageRange { min: 3.0, max: 5.0, unit: "g".to_string() }) },
        CatalogItem { name: "Melatonin".to_string(), category: "sleep".to_string(), dosage_range: Some(DosageRange { min: 0.5, max: 5.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Curcumin".to_string(), category: "anti-inflammatory".to_string(), dosage_range: Some(DosageRange { min: 500.0, max: 1000.0, unit: "mg".to_string() }) },
        CatalogItem { name: "NAC".to_string(), category: "antioxidant".to_string(), dosage_range: Some(DosageRange { min: 600.0, max: 1200.0, unit: "mg".to_string() }) },
        CatalogItem { name: "CoQ10".to_string(), category: "mitochondrial".to_string(), dosage_range: Some(DosageRange { min: 100.0, max: 200.0, unit: "mg".to_string() }) },
        CatalogItem { name: "Resveratrol".to_string(), category: "longevity".to_string(), dosage_range: Some(DosageRange { min: 100.0, max: 500.0, unit: "mg".to_string() }) },
    ]
}

pub fn log_page() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div>
            <h2>"Log Consumption"</h2>
            <p>"Search and log your supplement intake."</p>
            <ul>
                {catalog.iter().map(|item| {
                    let name = item.name.clone();
                    let dosage = item.dosage_range.as_ref()
                        .map(|d| format!("{} {}", d.min, d.unit))
                        .unwrap_or_default();
                    view! {
                        <li>{format!("{} - {}", name, dosage)}</li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

pub fn history_page() -> impl IntoView {
    view! {
        <div>
            <h2>"History"</h2>
            <p>"Coming soon..."</p>
        </div>
    }
}

pub fn vitals_page() -> impl IntoView {
    view! {
        <div>
            <h2>"Vitals"</h2>
            <p>"Coming soon..."</p>
        </div>
    }
}

pub fn stacks_page() -> impl IntoView {
    view! {
        <div>
            <h2>"Stacks"</h2>
            <p>"Coming soon..."</p>
        </div>
    }
}

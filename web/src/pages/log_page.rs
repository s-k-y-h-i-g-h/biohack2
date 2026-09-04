use leptos::*;
use leptos::prelude::*;
use engine::catalog::seed_catalog;

#[component]
pub fn LogPage() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div>
            <h2>"Log Consumption"</h2>
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

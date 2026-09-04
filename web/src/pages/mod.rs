use leptos::*;
use leptos::prelude::*;
use engine::catalog::seed_catalog;

pub fn log_page() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div>
            <h2>"Log Consumption"</h2>
            <ul>
                {catalog.iter().map(|item| {
                    view! {
                        <li>{format!("{} - {} {}", item.name, item.dosage_range.as_ref().map(|d| d.min).unwrap_or(0.0), item.dosage_range.as_ref().map(|d| d.unit.clone()).unwrap_or_default())}</li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

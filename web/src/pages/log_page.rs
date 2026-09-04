use leptos::*;
use engine::catalog::seed_catalog;

#[component]
pub fn log_page() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div>
            <h2>"Log Consumption"</h2>
            <ul>
                {catalog.iter().map(|item| {
                    let name = item.name.clone();
                    let dosage = format!(
                        "{} {}",
                        item.dosage_range.as_ref().map(|d| d.min.to_string()).unwrap_or_default(),
                        item.dosage_range.as_ref().map(|d| d.unit.clone()).unwrap_or_default()
                    );
                    view! {
                        <li>{format!("{} - {}", name, dosage)}</li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

#[component]
pub fn history_page() -> impl IntoView {
    view! {
        <div>
            <h2>"History"</h2>
            <p>"No entries yet."</p>
        </div>
    }
}

use leptos::*;
use leptos::prelude::*;
use engine::catalog::seed_catalog;

#[component]
pub fn LogPage() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div>
            <h2>"Log Consumption"</h2>
            <p>"Search and log your supplement intake."</p>
            <ul>
                {catalog.iter().map(|item| {
                    view! {
                        <li>
                            <strong>{&item.name}</strong>
                            " - "
                            {item.dosage_range.as_ref()
                                .map(|d| format!("{} {}", d.min, d.unit))
                                .unwrap_or_default()}
                        </li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

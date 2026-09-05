use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;

#[component]
pub fn SummaryStats(
    entries: Vec<LogEntry>,
) -> impl IntoView {
    let total_entries = entries.len();

    // Count by category
    let mut category_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for entry in &entries {
        let cat = match entry.item_type {
            engine::models::ItemType::Supplement => "Supplement",
            engine::models::ItemType::Medication => "Medication",
            engine::models::ItemType::Drug => "Drug",
            engine::models::ItemType::Food => "Food",
            engine::models::ItemType::Action => "Action",
        };
        *category_counts.entry(cat.to_string()).or_insert(0) += 1;
    }

    let sup_count = *category_counts.get("Supplement").unwrap_or(&0);
    let med_count = *category_counts.get("Medication").unwrap_or(&0);
    let food_count = *category_counts.get("Food").unwrap_or(&0);
    let action_count = *category_counts.get("Action").unwrap_or(&0);

    view! {
        <div class="summary-stats">
            <h3>"Summary"</h3>
            <div class="stat-row">
                <span>"Total Entries:"</span>
                <span class="stat-value">{total_entries}</span>
            </div>
            <div class="stat-row">
                <span>"Supplements:"</span>
                <span class="stat-value supplement">{sup_count}</span>
            </div>
            <div class="stat-row">
                <span>"Medications:"</span>
                <span class="stat-value medication">{med_count}</span>
            </div>
            <div class="stat-row">
                <span>"Food:"</span>
                <span class="stat-value food">{food_count}</span>
            </div>
            <div class="stat-row">
                <span>"Actions:"</span>
                <span class="stat-value action">{action_count}</span>
            </div>
        </div>
    }
}

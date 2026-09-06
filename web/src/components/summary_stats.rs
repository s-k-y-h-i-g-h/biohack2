use leptos::*;
use leptos::prelude::*;
use engine::models::*;
use crate::types::HistoryEntry;

#[component]
pub fn SummaryStats(
    entries: Vec<HistoryEntry>,
) -> impl IntoView {
    let total_entries = entries.len();

    // Count by category
    let mut category_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for entry in &entries {
        if let Some(cat) = entry.category() {
            *category_counts.entry(cat.to_string()).or_insert(0) += 1;
        }
    }

    let supplement_count = *category_counts.get("supplement").unwrap_or(&0);
    let medication_count = *category_counts.get("medication").unwrap_or(&0);
    let drug_count = *category_counts.get("drug").unwrap_or(&0);
    let food_count = *category_counts.get("food").unwrap_or(&0);
    let action_count = *category_counts.get("action").unwrap_or(&0);
    let vitals_count = *category_counts.get("vitals").unwrap_or(&0);

    view! {
        <div class="summary-stats">
            <h3>"Summary"</h3>
            <div class="stat-row">
                <span>"Total Entries:"</span>
                <span class="stat-value">{total_entries}</span>
            </div>
            <div class="stat-row">
                <span>"Supplements:"</span>
                <span class="stat-value supplement">{supplement_count}</span>
            </div>
            <div class="stat-row">
                <span>"Medications:"</span>
                <span class="stat-value medication">{medication_count}</span>
            </div>
            <div class="stat-row">
                <span>"Drugs:"</span>
                <span class="stat-value drug">{drug_count}</span>
            </div>
            <div class="stat-row">
                <span>"Food:"</span>
                <span class="stat-value food">{food_count}</span>
            </div>
            <div class="stat-row">
                <span>"Actions:"</span>
                <span class="stat-value action">{action_count}</span>
            </div>
            <div class="stat-row">
                <span>"Vitals:"</span>
                <span class="stat-value vitals">{vitals_count}</span>
            </div>
        </div>
    }
}

use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;
use crate::components::{HistoryView, SummaryStats};
use crate::state::db::get_log_entries;

fn filter_entries(entries: Vec<LogEntry>, search: &str, category: &Option<String>) -> Vec<LogEntry> {
    entries.into_iter().filter(|entry| {
        // Category filter
        if let Some(cat) = category {
            let entry_cat = match entry.item_type {
                engine::models::ItemType::Supplement => "supplement",
                engine::models::ItemType::Medication => "medication",
                engine::models::ItemType::Drug => "drug",
                engine::models::ItemType::Food => "food",
                engine::models::ItemType::Action => "action",
            };
            if entry_cat != cat.as_str() {
                return false;
            }
        }
        
        // Search filter
        if !search.is_empty() {
            let q = search.to_lowercase();
            if !entry.name.to_lowercase().contains(&q) {
                return false;
            }
        }
        
        true
    }).collect()
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);
    
    // Use Memo for reactive filtering
    let filtered_entries = create_memo(move |_| {
        let search_val = search.get();
        let category_val = category.get();
        let entries = get_log_entries().unwrap_or_default();
        filter_entries(entries, &search_val, &category_val)
    });

    view! {
        <div class="page">
            <h2>"History"</h2>
            <div class="filter-bar">
                <input
                    type="text"
                    placeholder="Search..."
                    prop:value=move || search.get()
                    on:input=move |e| {
                        search.set(event_target_value(&e));
                    }
                    class="search-input"
                    aria-label="Search entries"
                />
                <div class="category-chips">
                    <button
                        type="button"
                        class=move || {
                            if category.get().is_none() {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(None);
                        }
                        aria-label="Filter by All"
                    >"All"</button>
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("supplement".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("supplement".to_string()));
                        }
                        aria-label="Filter by Supplement"
                    >"Supplement"</button>
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("medication".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("medication".to_string()));
                        }
                        aria-label="Filter by Medication"
                    >"Medication"</button>
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("drug".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("drug".to_string()));
                        }
                        aria-label="Filter by Drug"
                    >"Drug"</button>
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("food".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("food".to_string()));
                        }
                        aria-label="Filter by Food"
                    >"Food"</button>
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("action".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("action".to_string()));
                        }
                        aria-label="Filter by Action"
                    >"Action"</button>
                </div>
            </div>
            <div class="history-container">
                <div class="history-list">
                    <HistoryView entries=filtered_entries.get() />
                </div>
                <div class="history-sidebar">
                    <SummaryStats entries=filtered_entries.get() />
                </div>
            </div>
        </div>
    }
}
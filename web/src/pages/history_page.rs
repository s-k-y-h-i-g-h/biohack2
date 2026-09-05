use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;
use crate::components::HistoryView;
use crate::state::db::get_log_entries;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let search = create_rw_signal(String::new());
    let category = create_rw_signal(None::<String>);
    
    view! {
        <div class="page">
            <h2>"History"</h2>
            <div class="filter-bar">
                <input
                    type="text"
                    placeholder="Search..."
                    prop:value=search
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
                    // Simple closure that reads signals - should auto-track
                    {move || {
                        let s = search.get();
                        let c = category.get();
                        let entries = get_log_entries().unwrap_or_default();
                        
                        let filtered: Vec<LogEntry> = entries.into_iter().filter(|entry| {
                            if let Some(cat) = &c {
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
                            
                            if !s.is_empty() {
                                let q = s.to_lowercase();
                                if !entry.name.to_lowercase().contains(&q) {
                                    return false;
                                }
                            }
                            
                            true
                        }).collect();
                        
                        view! {
                            <HistoryView entries=filtered />
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
use leptos::*;
use leptos::prelude::*;
use crate::state::db::get_log_entries;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);

    view! {
        <div class="page">
            <h2>"History"</h2>
            <div class="filter-bar">
                <input
                    type="text"
                    placeholder="Search..."
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
                    {move || {
                        let s = search.get();
                        let c = category.get();
                        let all_entries = get_log_entries().unwrap_or_default();
                        let filtered: Vec<_> = all_entries.into_iter()
                            .filter(|entry| {
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
                            })
                            .collect();

                        // Group entries by date
                        let mut grouped: std::collections::HashMap<String, Vec<engine::models::LogEntry>> = std::collections::HashMap::new();
                        for entry in &filtered {
                            let date = entry.timestamp.format("%Y-%m-%d").to_string();
                            grouped.entry(date).or_default().push(entry.clone());
                        }

                        let mut dates: Vec<String> = grouped.keys().cloned().collect();
                        dates.sort();
                        dates.reverse();

                        dates.iter().map(|date| {
                            let date_entries = grouped.get(date).cloned().unwrap_or_default();
                            let date_str = date.clone();
                            view! {
                                <div class="date-group">
                                    <h3 class="date-header">{date_str}</h3>
                                    {date_entries.into_iter().map(|entry| {
                                        let qty_str = if let Some(qty) = entry.quantity {
                                            format!("{} {}", qty, entry.unit.clone().unwrap_or_default())
                                        } else {
                                            String::new()
                                        };
                                        let time = entry.timestamp.format("%H:%M").to_string();
                                        let name = entry.name.clone();
                                        view! {
                                            <div class="entry-card">
                                                <div class="entry-time">{time}</div>
                                                <div class="entry-info">
                                                    <span class="entry-name">{name}</span>
                                                    <span class="entry-quantity">{qty_str}</span>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
        </div>
    }
}
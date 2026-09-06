use leptos::*;
use leptos::prelude::*;
use crate::state::db::{get_log_entries, get_vitals_entries};
use crate::components::SummaryStats;
use crate::types::HistoryEntry;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);

    // Computed filtered entries (used by both SummaryStats and the list)
    let filtered_entries = move || {
        let s = search.get();
        let c = category.get();
        let log_entries = get_log_entries().unwrap_or_default();
        let vitals_entries = get_vitals_entries(&Default::default()).unwrap_or_default();

        let mut all_entries: Vec<HistoryEntry> = log_entries
            .into_iter()
            .map(HistoryEntry::Log)
            .chain(vitals_entries.into_iter().map(HistoryEntry::Vitals))
            .collect();

        // Sort by timestamp descending
        all_entries.sort_by(|a, b| b.timestamp().cmp(&a.timestamp()));

        // Apply filters
        let filtered: Vec<HistoryEntry> = all_entries
            .into_iter()
            .filter(|entry| {
                if let Some(cat) = &c {
                    if let Some(entry_cat) = entry.category() {
                        if entry_cat != *cat {
                            return false;
                        }
                    }
                }
                if !s.is_empty() {
                    let q = s.to_lowercase();
                    if !entry.name().to_lowercase().contains(&q) {
                        return false;
                    }
                }
                true
            })
            .collect();

        filtered
    };

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
                    <button
                        type="button"
                        class=move || {
                            if category.get() == Some("vitals".to_string()) {
                                "chip active"
                            } else {
                                "chip"
                            }
                        }
                        on:click=move |_| {
                            category.set(Some("vitals".to_string()));
                        }
                        aria-label="Filter by Vitals"
                    >"Vitals"</button>
                </div>
                <button
                    type="button"
                    class="export-btn"
                    aria-label="Export data"
                    on:click=move |_| {
                        let _ = crate::state::db::export_data();
                    }
                >"Export CSV"</button>
            </div>
            <SummaryStats entries=filtered_entries() />
            <div class="history-container">
                <div class="history-list">
                    {move || {
                        let filtered = filtered_entries();

                        // Group by date
                        let mut grouped: std::collections::HashMap<String, Vec<HistoryEntry>> = std::collections::HashMap::new();
                        for entry in &filtered {
                            let date = entry.timestamp().format("%Y-%m-%d").to_string();
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
                                        let time = entry.timestamp().format("%H:%M").to_string();
                                        let name = entry.name();
                                        let note_text = match &entry {
                                            HistoryEntry::Log(log_entry) => log_entry.notes.clone(),
                                            HistoryEntry::Vitals(_) => None,
                                        };
                                        let is_vitals = matches!(entry, HistoryEntry::Vitals(_));

                                        view! {
                                            <div class=format!("entry-card{}", if is_vitals { " entry-card--vitals" } else { "" })>
                                                <div class="entry-time">{time}</div>
                                                <div class="entry-info">
                                                    <span class="entry-name">{name}</span>
                                                    {move || note_text.clone().map(|n| {
                                                        view! {
                                                            <span class="entry-note">{n}</span>
                                                        }
                                                    })}
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

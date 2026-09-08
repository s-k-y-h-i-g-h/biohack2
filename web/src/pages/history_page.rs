use leptos::prelude::*;
use crate::state::db::{get_log_entries, get_vitals_entries, get_notes};
use crate::state::store::AppContext;
use crate::components::SummaryStats;
use crate::types::HistoryEntry;

const PAGE_SIZE: usize = 100;

#[component]
pub fn HistoryPage() -> impl IntoView {
    // Global data version — re-reads storage after any page writes
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let search = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);
    let date_start = RwSignal::new(String::new()); // YYYY-MM-DD, inclusive
    let date_end = RwSignal::new(String::new());   // YYYY-MM-DD, inclusive
    let visible = RwSignal::new(PAGE_SIZE);         // pagination: entries shown

    // Computed filtered entries (used by both SummaryStats and the list)
    let filtered_entries = move || {
        let _ = version.get(); // track: re-read after writes on any page
        let s = search.get();
        let c = category.get();
        let start = date_start.get();
        let end = date_end.get();

        let log_entries = get_log_entries().unwrap_or_default();
        let vitals_entries = get_vitals_entries(&Default::default()).unwrap_or_default();
        let note_entries = get_notes().unwrap_or_default();

        let mut all_entries: Vec<HistoryEntry> = log_entries
            .into_iter()
            .map(HistoryEntry::Log)
            .chain(vitals_entries.into_iter().map(HistoryEntry::Vitals))
            .chain(note_entries.into_iter().map(HistoryEntry::Note))
            .collect();

        // Sort by timestamp descending
        all_entries.sort_by(|a, b| b.timestamp().cmp(&a.timestamp()));

        // Apply filters
        all_entries.into_iter().filter(|entry| {
            // Category filter
            if let Some(cat) = &c {
                if let Some(entry_cat) = entry.category() {
                    if &entry_cat != cat {
                        return false;
                    }
                }
            }

            // Date-range filter (inclusive, compares the date part)
            let date = entry.timestamp().format("%Y-%m-%d").to_string();
            if !start.is_empty() && date.as_str() < start.as_str() {
                return false;
            }
            if !end.is_empty() && date.as_str() > end.as_str() {
                return false;
            }

            // Search filter: name or note content
            if !s.is_empty() {
                let q = s.to_lowercase();
                let name_matches = entry.name().to_lowercase().contains(&q);
                let notes_matches = match entry {
                    HistoryEntry::Log(log) => log.notes.as_ref().map(|n| n.to_lowercase().contains(&q)).unwrap_or(false),
                    HistoryEntry::Vitals(v) => v.notes.as_ref().map(|n| n.to_lowercase().contains(&q)).unwrap_or(false),
                    HistoryEntry::Note(n) => n.content.to_lowercase().contains(&q),
                };
                if !name_matches && !notes_matches {
                    return false;
                }
            }
            true
        }).collect::<Vec<_>>()
    };

    // Filtered entries visible on the current page
    let visible_entries = move || {
        let n = visible.get();
        filtered_entries().into_iter().take(n).collect::<Vec<_>>()
    };

    let total_count = move || filtered_entries().len();
    let shown_count = move || visible_entries().len();

    let chips = [
        ("All", ""),
        ("Supplement", "supplement"),
        ("Medication", "medication"),
        ("Drug", "drug"),
        ("Food", "food"),
        ("Action", "action"),
        ("Vitals", "vitals"),
        ("Note", "note"),
    ];

    view! {
        <div class="page">
            <h2>"History"</h2>
            <div class="filter-bar">
                <input
                    type="text"
                    placeholder="Search... (names and notes)"
                    on:input=move |e| { search.set(event_target_value(&e)); }
                    class="search-input"
                    aria-label="Search entries"
                />
                <div class="date-range">
                    <label for="date-start">"From"</label>
                    <input
                        id="date-start"
                        type="date"
                        on:input=move |e| { date_start.set(event_target_value(&e)); }
                        aria-label="Filter from date"
                    />
                    <label for="date-end">"To"</label>
                    <input
                        id="date-end"
                        type="date"
                        on:input=move |e| { date_end.set(event_target_value(&e)); }
                        aria-label="Filter to date"
                    />
                    <button
                        type="button"
                        class="chip"
                        on:click=move |_| {
                            date_start.set(String::new());
                            date_end.set(String::new());
                        }
                        aria-label="Clear date range"
                    >"Clear dates"</button>
                </div>
                <div class="category-chips">
                    {chips.iter().map(|(label, value)| {
                        let value = value.to_string();
                        let value2 = value.clone();
                        let label = label.to_string();
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let active = if value.is_empty() {
                                        category.get().is_none()
                                    } else {
                                        category.get().as_deref() == Some(value.as_str())
                                    };
                                    if active { "chip active" } else { "chip" }
                                }
                                on:click=move |_| {
                                    category.set(if value2.is_empty() { None } else { Some(value2.clone()) });
                                }
                                aria-label=format!("Filter by {}", label)
                            >{label.clone()}</button>
                        }
                    }).collect_view()}
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
                        let filtered = visible_entries();

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
                                        let details = entry.details();
                                        let note_text = match &entry {
                                            HistoryEntry::Log(log_entry) => log_entry.notes.clone(),
                                            HistoryEntry::Vitals(_) => None,
                                            HistoryEntry::Note(n) => Some(n.content.clone()),
                                        };
                                        let is_vitals = matches!(entry, HistoryEntry::Vitals(_));
                                        let is_note = matches!(entry, HistoryEntry::Note(_));

                                        view! {
                                            <div class=format!("entry-card{}{}",
                                                if is_vitals { " entry-card--vitals" } else { "" },
                                                if is_note { " entry-card--note" } else { "" })>
                                                <div class="entry-time">{time}</div>
                                                <div class="entry-info">
                                                    <span class="entry-name">{name}</span>
                                                    {move || details.clone().map(|d| {
                                                        view! {
                                                            <span class="entry-quantity">{d}</span>
                                                        }
                                                    })}
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
                    <Show when=move || { let t = total_count(); let s = shown_count(); t > s }>
                        <div class="pagination-row">
                            <span class="pagination-info">
                                {move || format!("Showing {} of {} entries", shown_count(), total_count())}
                            </span>
                            <button
                                type="button"
                                class="load-more-btn"
                                on:click=move |_| {
                                    visible.update(|n| *n += PAGE_SIZE);
                                }
                                aria-label="Load more entries"
                            >"Load more"</button>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
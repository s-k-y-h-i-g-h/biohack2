use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;

#[component]
pub fn HistoryView(
    entries: Vec<LogEntry>,
) -> impl IntoView {
    // Group entries by date
    let mut grouped: std::collections::HashMap<String, Vec<LogEntry>> = std::collections::HashMap::new();
    for entry in &entries {
        let date = entry.timestamp.format("%Y-%m-%d").to_string();
        grouped.entry(date).or_default().push(entry.clone());
    }

    let mut dates: Vec<String> = grouped.keys().cloned().collect();
    dates.sort();
    dates.reverse();

    view! {
        <div class="history-view">
            {move || {
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
    }
}

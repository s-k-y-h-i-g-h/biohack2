use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;

#[component]
pub fn TimelineView(
    entries: Vec<LogEntry>,
) -> impl IntoView {
    let total_entries = entries.len();
    let last_7 = entries.iter()
        .filter(|e| {
            let now = chrono::Utc::now();
            let days_diff = (now.date_naive() - e.timestamp.date_naive()).num_days();
            days_diff <= 7
        })
        .count();

    view! {
        <div class="timeline-view">
            <div class="timeline-stats">
                <span>"Total: " {total_entries}</span>
                <span>"Last 7 days: " {last_7}</span>
            </div>
            <div class="timeline-line">
                {entries.iter().take(20).enumerate().map(|(i, entry)| {
                    let time = entry.timestamp.format("%H:%M").to_string();
                    let day = entry.timestamp.format("%b %d").to_string();
                    let color = match entry.item_type {
                        engine::models::ItemType::Supplement => "#4ade80",
                        engine::models::ItemType::Medication => "#60a5fa",
                        engine::models::ItemType::Drug => "#f87171",
                        engine::models::ItemType::Food => "#fbbf24",
                        engine::models::ItemType::Action => "#c084fc",
                    };

                    view! {
                        <div class="timeline-item">
                            <div class="timeline-dot" style=format!("background-color: {}", color)>
                            </div>
                            <div class="timeline-content">
                                <div class="timeline-date">{day}</div>
                                <div class="timeline-time">{time}</div>
                                <div class="timeline-name">{entry.name.clone()}</div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

use crate::state::db::{
    create_log_entry, get_log_entries, get_notes, get_vitals_entries, search_catalog,
};
use crate::state::store::AppContext;
use crate::types::HistoryEntry;
use engine::models::LogEntry;
use engine::safety::SafetyEngine;
use leptos::prelude::*;

#[component]
pub fn LogPage() -> impl IntoView {
    let interaction_warning = RwSignal::new(None::<LogEntry>);
    // Persisted catalog (localStorage) — stable IDs, seeded on first use.
    // seed_catalog() regenerates UUIDs per call; never use it for saved references.
    let catalog = search_catalog("").unwrap_or_default();

    let handle_save = move |entry: LogEntry| {
        // Check for interactions before saving
        let engine = SafetyEngine::new();
        let existing_entries = get_log_entries().unwrap_or_default();
        let warnings = engine.check_interactions(&entry, &existing_entries);

        if !warnings.is_empty() {
            // Store the entry for potential save after acknowledgment
            interaction_warning.set(Some(entry));
            return;
        }

        // No warnings, save the entry
        if let Err(e) = create_log_entry(&entry) {
            eprintln!("Failed to save entry: {}", e);
        }
    };

    let handle_acknowledge_and_save = move |_| {
        if let Some(entry) = interaction_warning.get() {
            // Save anyway after acknowledgment
            if let Err(e) = create_log_entry(&entry) {
                eprintln!("Failed to save entry: {}", e);
            }
        }
        interaction_warning.set(None);
    };

    // ── Recent history sidebar ──────────────────────────────────────────────
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let recent_entries = move || {
        version.get(); // track
        let log_entries = get_log_entries().unwrap_or_default();
        let vitals_entries = get_vitals_entries(&Default::default()).unwrap_or_default();
        let note_entries = get_notes().unwrap_or_default();

        let mut all: Vec<HistoryEntry> = log_entries
            .into_iter()
            .map(HistoryEntry::Log)
            .chain(vitals_entries.into_iter().map(HistoryEntry::Vitals))
            .chain(note_entries.into_iter().map(HistoryEntry::Note))
            .collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.timestamp()));
        all.into_iter().take(5).collect::<Vec<_>>()
    };

    view! {
        <div class="page log-page-layout">
            <div class="log-page-main">
                <h2>"Log Consumption"</h2>
                <crate::components::LogForm
                    catalog=catalog
                    on_save=Callback::new(handle_save)
                />

                <Show when=move || interaction_warning.get().is_some()>
                    <div class="interaction-warning-container">
                        <p>"Interaction warning detected. Please review before saving."</p>
                        <button
                            type="button"
                            class="acknowledge-btn"
                            on:click=handle_acknowledge_and_save
                            aria-label="Acknowledge and save anyway"
                        >"Acknowledge and Save Anyway"</button>
                    </div>
                </Show>
            </div>

            <aside class="log-page-sidebar">
                <div class="summary-stats">
                    <h3>"Recent Activity"</h3>
                    <Show when=move || recent_entries().is_empty()>
                        <p class="empty-state">"No entries yet. Log your first item above."</p>
                    </Show>
                    {move || {
                        recent_entries().into_iter().map(|entry| {
                            let time = entry.timestamp().with_timezone(&chrono::Local).format("%H:%M").to_string();
                            let name = entry.name();
                            let details = entry.details();
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
                                            view! { <span class="entry-quantity">{d}</span> }
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </aside>
        </div>
    }
}

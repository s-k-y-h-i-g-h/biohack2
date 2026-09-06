use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;
use engine::safety::SafetyEngine;
use crate::state::db::{create_log_entry, get_log_entries};

#[component]
pub fn LogPage() -> impl IntoView {
    let interaction_warning = RwSignal::new(None::<LogEntry>);
    let catalog = engine::catalog::seed_catalog();

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

    view! {
        <div class="page">
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
    }
}

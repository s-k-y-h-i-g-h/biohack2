use leptos::*;
use leptos::prelude::*;
use engine::models::{LogEntry, ItemType};
use crate::state::db::update_log_entry;

#[component]
pub fn NoteInput(
    entry: LogEntry,
    on_updated: Callback<()>,
) -> impl IntoView {
    let is_editing = RwSignal::new(false);
    let note_text = RwSignal::new(entry.notes.clone().unwrap_or_default());
    let is_saving = RwSignal::new(false);
    let has_note = entry.notes.is_some();
    let entry_id = entry.id;
    let entry_user_id = RwSignal::new(entry.user_id.clone());
    let entry_name = RwSignal::new(entry.name.clone());
    let entry_timestamp = entry.timestamp;
    let original_notes = RwSignal::new(entry.notes.clone());

    view! {
        <div class="note-input">
            <Show when=move || !is_editing.get()>
                <Show when=move || has_note>
                    <div class="note-display" on:click=move |_| { is_editing.set(true); }>
                        <span class="note-text">{move || note_text.get()}</span>
                        <span class="note-edit-hint">Edit</span>
                    </div>
                </Show>
                <Show when=move || !has_note>
                    <button
                        type="button"
                        class="add-note-btn"
                        on:click=move |_| { is_editing.set(true); }
                        aria-label="Add note"
                    >"+ Add Note"</button>
                </Show>
            </Show>
            <Show when=move || is_editing.get()>
                <div class="note-editor">
                    <textarea
                        placeholder="Add a note..."
                        aria-label="Note text"
                        on:input=move |e| { note_text.set(event_target_value(&e)); }
                    >{move || note_text.get()}</textarea>
                    <div class="note-actions">
                        <button
                            type="button"
                            class="save-note-btn"
                            disabled=move || is_saving.get()
                            on:click=move |_| {
                                is_saving.set(true);
                                let text = note_text.get();
                                let uid = entry_user_id.get();
                                let name = entry_name.get();
                                let mut e = LogEntry {
                                    id: entry_id,
                                    user_id: uid,
                                    item_type: ItemType::default(),
                                    item_id: None,
                                    name,
                                    quantity: None,
                                    unit: None,
                                    route: None,
                                    timestamp: entry_timestamp,
                                    stack_id: None,
                                    notes: if text.is_empty() { None } else { Some(text) },
                                    acknowledged_interaction: false,
                                    custom_fields: None,
                                };

                                if update_log_entry(&e).is_ok() {
                                    is_editing.set(false);
                                    on_updated.run(());
                                }
                                is_saving.set(false);
                            }
                            aria-label="Save note"
                        >{move || if is_saving.get() { "Saving..." } else { "Save" }}</button>
                        <button
                            type="button"
                            class="cancel-note-btn"
                            on:click=move |_| {
                                note_text.set(original_notes.get().clone().unwrap_or_default());
                                is_editing.set(false);
                            }
                            aria-label="Cancel note editing"
                        >"Cancel"</button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

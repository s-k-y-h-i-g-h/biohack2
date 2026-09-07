use leptos::prelude::*;
use engine::models::Note;
use crate::state::db::{create_note, get_notes, update_note, delete_note};
use crate::components::NoteForm;

/// Notes page — log standalone notes (realizations, observations) and review recent ones.
/// Notes are first-class entries: they also appear in the unified history view.
#[component]
pub fn NotesPage() -> impl IntoView {
    let notes = RwSignal::new(Vec::<Note>::new());
    let message = RwSignal::new(None::<String>);
    let editing_id = RwSignal::new(None::<uuid::Uuid>);
    let edit_content = RwSignal::new(String::new());

    let load_notes = move || get_notes().unwrap_or_default();
    notes.set(load_notes());

    let flash = move |msg: String| {
        message.set(Some(msg));
        set_timeout(
            move || message.set(None),
            std::time::Duration::from_millis(2000),
        );
    };

    let handle_save = move |note: Note| {
        match create_note(&note) {
            Ok(()) => {
                notes.set(load_notes());
                flash("Note logged!".to_string());
            }
            Err(e) => flash(format!("Failed to save note: {}", e)),
        }
    };

    let start_edit = move |note: Note| {
        editing_id.set(Some(note.id));
        edit_content.set(note.content);
    };

    let handle_edit_save = move |_| {
        if let Some(id) = editing_id.get_untracked() {
            if let Some(mut note) = notes.get_untracked().into_iter().find(|n| n.id == id) {
                let text = edit_content.get_untracked();
                if text.trim().is_empty() {
                    flash("Note cannot be empty".to_string());
                    return;
                }
                note.content = text;
                note.timestamp = chrono::Utc::now();
                match update_note(&note) {
                    Ok(()) => {
                        notes.set(load_notes());
                        editing_id.set(None);
                        edit_content.set(String::new());
                        flash("Note updated".to_string());
                    }
                    Err(e) => flash(format!("Failed to update note: {}", e)),
                }
            }
        }
    };

    let handle_cancel_edit = move |_| {
        editing_id.set(None);
        edit_content.set(String::new());
    };

    let handle_delete = move |id: uuid::Uuid| {
        match delete_note(&id.to_string()) {
            Ok(()) => {
                notes.set(load_notes());
                flash("Note deleted".to_string());
            }
            Err(e) => flash(format!("Failed to delete note: {}", e)),
        }
    };

    view! {
        <div class="page">
            <h2>"Notes"</h2>

            <Show when=move || message.get().is_some()>
                <div class="toast success" role="status">
                    {move || message.get()}
                </div>
            </Show>

            <div class="notes-form-section">
                <h3>"Log a Note"</h3>
                <NoteForm on_save=Callback::new(handle_save) />
            </div>

            <div class="notes-list-section">
                <h3>"Recent Notes"</h3>
                <Show when=move || notes.get().is_empty()>
                    <p class="empty-state">"No notes yet. Log your first realization above."</p>
                </Show>
                <div class="notes-list">
                    {move || {
                        // Read both signals so the list re-renders on edit-state change too
                        let editing = editing_id.get();
                        notes.get().into_iter().map(move |note| {
                            let id = note.id;
                            let is_editing = editing == Some(id);
                            let time = note.timestamp.format("%Y-%m-%d %H:%M").to_string();
                            let content = note.content.clone();

                            if is_editing {
                                view! {
                                    <div class="note-card note-card--editing">
                                        <div class="note-edit-row">
                                            <textarea
                                                aria-label="Edit note"
                                                rows="3"
                                                on:input=move |e| { edit_content.set(event_target_value(&e)); }
                                            >{move || edit_content.get()}</textarea>
                                            <div class="note-actions">
                                                <button
                                                    type="button"
                                                    class="save-note-btn"
                                                    on:click=handle_edit_save
                                                    aria-label="Save edited note"
                                                >"Save"</button>
                                                <button
                                                    type="button"
                                                    class="cancel-note-btn"
                                                    on:click=handle_cancel_edit
                                                    aria-label="Cancel editing"
                                                >"Cancel"</button>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="note-card">
                                        <div class="note-content-row">
                                            <span class="note-timestamp">{time}</span>
                                            <p class="note-text">{content}</p>
                                        </div>
                                        <div class="note-actions">
                                            <button
                                                type="button"
                                                class="edit-note-btn"
                                                on:click=move |_| { start_edit(note.clone()); }
                                                aria-label="Edit note"
                                            >"Edit"</button>
                                            <button
                                                type="button"
                                                class="delete-note-btn"
                                                on:click=move |_| { handle_delete(id); }
                                                aria-label="Delete note"
                                            >"Delete"</button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
        </div>
    }
}
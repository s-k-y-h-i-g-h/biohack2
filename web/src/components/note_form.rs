use engine::models::Note;
use leptos::prelude::*;
use uuid::Uuid;

/// Standalone note logging form — textarea + optional link to a recent log entry.
/// The note is saved as a first-class entry with its own timestamp.
#[component]
pub fn NoteForm(on_save: Callback<Note>) -> impl IntoView {
    let content = RwSignal::new(String::new());
    let linked_entry_id = RwSignal::new(None::<Uuid>);
    let show_link = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    // Recent log entries for optional context linking
    let recent_entries = RwSignal::new(Vec::<(Uuid, String)>::new());
    recent_entries.set(
        crate::state::db::get_log_entries()
            .unwrap_or_default()
            .into_iter()
            .take(10)
            .map(|e| (e.id, e.name))
            .collect(),
    );

    let handle_save = move |_| {
        let text = content.get();
        if text.trim().is_empty() {
            error.set(Some("Note cannot be empty".to_string()));
            return;
        }
        error.set(None);

        let note = Note {
            id: Uuid::new_v4(),
            user_id: "local-device".to_string(),
            content: text,
            timestamp: chrono::Utc::now(),
            linked_entry_id: linked_entry_id.get(),
        };

        on_save.run(note);
        content.set(String::new());
        linked_entry_id.set(None);
        show_link.set(false);
    };

    view! {
        <div class="note-form">
            <textarea
                placeholder="Log a realization or observation..."
                aria-label="Note text"
                rows="4"
                on:input=move |e| { content.set(event_target_value(&e)); }
            >{move || content.get()}</textarea>

            <div class="note-form-actions">
                <button
                    type="button"
                    class="link-toggle-btn"
                    on:click=move |_| { show_link.update(|v| *v = !*v); }
                    aria-label="Toggle link to log entry"
                >
                    {move || if show_link.get() { "− Remove link" } else { "+ Link to log entry" }}
                </button>
                <button
                    type="button"
                    class="save-button"
                    on:click=handle_save
                    aria-label="Save note"
                >"Save Note"</button>
            </div>

            <Show when=move || show_link.get()>
                <div class="note-link-section">
                    <label for="note-link">"Link to recent entry (optional):"</label>
                    <select
                        id="note-link"
                        aria-label="Link note to log entry"
                        on:change=move |e| {
                            let val = event_target_value(&e);
                            linked_entry_id.set(if val.is_empty() {
                                None
                            } else {
                                val.parse().ok()
                            });
                        }
                    >
                        <option value="">"— None —"</option>
                        {move || recent_entries.get().into_iter().map(|(id, name)| {
                            let val = id.to_string();
                            view! {
                                <option value=val.clone()>{format!("{} ({})", name, id)}</option>
                            }
                        }).collect_view()}
                    </select>
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">
                    {move || error.get()}
                </div>
            </Show>
        </div>
    }
}

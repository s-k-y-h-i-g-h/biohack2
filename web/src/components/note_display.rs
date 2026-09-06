use leptos::*;
use leptos::prelude::*;
use engine::models::LogEntry;

#[component]
pub fn NoteDisplay(entry: LogEntry) -> impl IntoView {
    let has_note = entry.notes.is_some();
    let note_text = entry.notes.clone().unwrap_or_default();

    view! {
        <Show when=move || has_note>
            <div class="note-display-inline">
                <span class="note-label">"Note:"</span>
                <span class="note-content">{note_text.clone()}</span>
            </div>
        </Show>
    }
}

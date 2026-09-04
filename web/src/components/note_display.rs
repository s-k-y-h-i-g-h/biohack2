use leptos::*;

#[component]
pub fn NoteDisplay(note: Signal<String>) -> impl IntoView {
    view! {
        <div class="note-display">
            <p>{note.get()}</p>
        </div>
    }
}

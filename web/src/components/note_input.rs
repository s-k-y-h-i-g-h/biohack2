use leptos::*;

#[component]
pub fn NoteInput() -> impl IntoView {
    let (text, set_text) = create_signal(String::new());
    let (editing, set_editing) = create_signal(false);
    
    view! {
        {editing.get().then(|| {
            view! {
                <div class="note-input">
                    <textarea
                        value=text
                        on:input=move |e| set_text.set(event_target_value(&e))
                        placeholder="Add a note..."
                        rows="2"
                    />
                    <button class="btn btn-small" on:click=move |_| set_editing.set(false)>"Save"</button>
                </div>
            }.into_view()
        })}
    }
}

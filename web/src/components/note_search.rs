use leptos::*;

#[component]
pub fn NoteSearch() -> impl IntoView {
    let (query, set_query) = create_signal(String::new());
    let (results, set_results) = create_signal(Vec::<String>::new());
    
    effect(move |_| {
        if !query.get().is_empty() {
            // Search notes
            set_results.set(vec![format!("Results for: {}", query.get()))]);
        }
    });
    
    view! {
        <div class="note-search">
            <input
                type="text"
                value=query
                on:input=move |e| set_query.set(event_target_value(&e))
                placeholder="Search notes..."
            />
            <div class="search-results">
                {results.get().iter().map(|r| {
                    view! {
                        <div class="search-result">{r}</div>
                    }.into_view()
                }).collect_view()}
            </div>
        </div>
    }
}

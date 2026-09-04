use leptos::*;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let (entries, set_entries) = create_signal(vec![]);
    let (filter, set_filter) = create_signal(String::new());
    
    // In production, load from SQLite via WASM
    effect(move |_| {
        leptos::logging::log!("Loading history with filter: {}", filter.get());
    });
    
    view! {
        <div class="page">
            <h1>"History"</h1>
            
            <div class="filters">
                <input
                    type="text"
                    placeholder="Filter entries..."
                    value=filter
                    on:input=move |e| set_filter.set(event_target_value(&e))
                    class="filter-input"
                />
            </div>
            
            <div class="entries-list">
                {if entries.is_empty() {
                    view! {
                        <div class="empty-state">
                            <p>"No entries yet. Start logging!"</p>
                        </div>
                    }.into_view()
                } else {
                    entries.get().iter().map(|entry| {
                        view! {
                            <div class="entry-card">
                                <div class="entry-header">
                                    <span class="entry-name">{entry.name}</span>
                                    <span class="entry-time">{format!("{}", entry.timestamp.format("%Y-%m-%d %H:%M"))}</span>
                                </div>
                                <div class="entry-details">
                                    <span class="entry-type">{entry.item_type.to_string()}</span>
                                    {entry.quantity.map(|q| view! {
                                        <span class="entry-quantity">{format!("{} {}", q, entry.unit.as_deref().unwrap_or(""))}</span>
                                    }.into_view())}
                                </div>
                                {entry.notes.as_ref().map(|notes| view! {
                                    <div class="entry-notes">{notes}</div>
                                }.into_view())}
                            </div>
                        }.into_view()
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

use leptos::*;

#[component]
pub fn HistoryView() -> impl IntoView {
    let (entries, set_entries) = create_signal(vec![]);
    let (filter, set_filter) = create_signal(String::new());
    let (category, set_category) = create_signal(String::new());
    
    view! {
        <div class="history-view">
            <div class="filters">
                <input
                    type="text"
                    placeholder="Search entries..."
                    value=filter
                    on:input=move |e| set_filter.set(event_target_value(&e))
                    class="filter-input"
                />
                <select value=category.on_change(move |v| set_category.set(v))>
                    <option value="">All Categories</option>
                    <option value="supplement">Supplements</option>
                    <option value="medication">Medications</option>
                    <option value="food">Food</option>
                    <option value="action">Actions</option>
                </select>
            </div>
            
            <div class="entries-list">
                {if entries.is_empty() {
                    view! {
                        <div class="empty-state">
                            <p>"No entries found. Start logging!"</p>
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
                                    <span class="entry-type">{format!("{:?}", entry.item_type)}</span>
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

use leptos::*;

#[component]
pub fn FilterBar() -> impl IntoView {
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    let (category, set_category) = create_signal(String::new());
    
    view! {
        <div class="filter-bar">
            <div class="date-range">
                <input type="date" value=start_date.on_change(move |v| set_start_date.set(v)) />
                <span>"to"</span>
                <input type="date" value=end_date.on_change(move |v| set_end_date.set(v)) />
            </div>
            <select value=category.on_change(move |v| set_category.set(v))>
                <option value="">All Categories</option>
                <option value="supplement">Supplements</option>
                <option value="medication">Medications</option>
                <option value="drug">Drugs</option>
                <option value="food">Food</option>
                <option value="action">Actions</option>
            </select>
        </div>
    }
}

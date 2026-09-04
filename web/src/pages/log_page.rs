use leptos::*;

#[component]
pub fn LogPage() -> impl IntoView {
    let (search, set_search) = create_signal(String::new());
    let (selected_item, set_selected_item) = create_signal::<Option<String>>(None);
    let (quantity, set_quantity) = create_signal(String::new());
    let (unit, set_unit) = create_signal(String::new());
    let (submitted, set_submitted) = create_signal(false);
    
    let catalog_items = create_async_resource(|| async {
        // In production, this would call the engine via WASM
        // For now, return empty
        leptos::logging::log!("Loading catalog");
        vec![]
    });
    
    let handle_submit = move |_| {
        if let Some(item) = selected_item.get() {
            leptos::logging::log!("Logging: {} {} {}", item, quantity.get(), unit.get());
            set_submitted.set(true);
            // Reset form
            set_selected_item.set(None);
            set_quantity.set(String::new());
            set_unit.set(String::new());
        }
    };
    
    view! {
        <div class="page">
            <h1>"Log Consumption"</h1>
            
            <div class="card">
                <h2>"Search Catalog"</h2>
                <input
                    type="text"
                    placeholder="Search supplements..."
                    value=search
                    on:input=move |e| set_search.set(event_target_value(&e))
                    class="search-input"
                />
                
                <div class="catalog-list">
                    {catalog_items.read().as_ref().map(|items| {
                        items.iter().map(|item| {
                            let item_name = item.name.clone();
                            view! {
                                <button
                                    class="catalog-item"
                                    class:selected=move || selected_item.get() == Some(item_name.clone())
                                    on:click=move |_| set_selected_item.set(Some(item_name.clone()))
                                >
                                    <span class="item-name">{item.name}</span>
                                    <span class="item-dosage">{format!("{} {}", item.dosage_range.as_ref().map(|d| d.min).unwrap_or(0.0), item.dosage_range.as_ref().map(|d| d.unit.clone()).unwrap_or_default())}</span>
                                </button>
                            }.into_view()
                        }).collect_view()
                    })}
                </div>
            </div>
            
            {selected_item.get().map(|item| {
                view! {
                    <div class="card">
                        <h2>{format!("Log: {}", item)}</h2>
                        <div class="form-group">
                            <label>"Quantity"</label>
                            <input
                                type="number"
                                value=quantity
                                on:input=move |e| set_quantity.set(event_target_value(&e))
                                placeholder="e.g., 5000"
                            />
                        </div>
                        <div class="form-group">
                            <label>"Unit"</label>
                            <input
                                type="text"
                                value=unit
                                on:input=move |e| set_unit.set(event_target_value(&e))
                                placeholder="e.g., IU, mg, min"
                            />
                        </div>
                        <button class="btn btn-primary" on:click=handle_submit>
                            "Log Entry"
                        </button>
                    </div>
                }.into_view()
            })}
            
            {submitted.get().then(|| view! {
                <div class="success-message">
                    "Entry logged successfully!"
                </div>
            })}
        </div>
    }
}

use leptos::*;

#[component]
pub fn LogForm() -> impl IntoView {
    let (search, set_search) = create_signal(String::new());
    let (selected_id, set_selected_id) = create_signal::<Option<String>>(None);
    let (quantity, set_quantity) = create_signal(String::new());
    let (unit, set_unit) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    view! {
        <div class="log-form">
            <h2>"Log Item"</h2>
            
            <div class="search-box">
                <input
                    type="text"
                    placeholder="Search catalog..."
                    value=search
                    on:input=move |e| set_search.set(event_target_value(&e))
                    class="search-input"
                />
            </div>
            
            <div class="catalog-results">
                // Catalog items would be loaded here via WASM call
                <p class="placeholder">"Search to find supplements..."</p>
            </div>
            
            {selected_id.get().map(|id| {
                view! {
                    <div class="selected-item">
                        <p>"Selected: {id}"</p>
                        <div class="dosage-inputs">
                            <input
                                type="number"
                                placeholder="Quantity"
                                value=quantity
                                on:input=move |e| set_quantity.set(event_target_value(&e))
                            />
                            <input
                                type="text"
                                placeholder="Unit (mg, IU, min)"
                                value=unit
                                on:input=move |e| set_unit.set(event_target_value(&e))
                            />
                        </div>
                        <button
                            class="btn btn-primary"
                            disabled=loading.get()
                            on:click=move |_| {
                                set_loading.set(true);
                                set_error.set(None);
                                // Submit logic here
                            }
                        >
                            {if loading.get() { "Saving..." } else { "Log Entry" }}
                        </button>
                    </div>
                }.into_view()
            })}
            
            {error.get().map(|err| {
                view! {
                    <div class="error-message">{err}</div>
                }.into_view()
            })}
        </div>
    }
}

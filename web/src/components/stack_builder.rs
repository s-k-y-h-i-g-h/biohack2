use leptos::*;

#[component]
pub fn StackBuilder() -> impl IntoView {
    let (stack_name, set_stack_name) = create_signal(String::new());
    let (items, set_items) = create_signal(Vec::<(String, String, String)>::new());
    
    view! {
        <div class="stack-builder">
            <h2>"Create Stack"</h2>
            <div class="form-group">
                <label>"Stack Name"</label>
                <input
                    type="text"
                    value=stack_name
                    on:input=move |e| set_stack_name.set(event_target_value(&e))
                    placeholder="e.g., Morning Protocol"
                />
            </div>
            
            <div class="items-list">
                {items.get().iter().enumerate().map(|(i, (item_id, qty, unit))| {
                    view! {
                        <div class="stack-item">
                            <input type="text" value=item_id.clone() placeholder="Item ID" />
                            <input type="text" value=qty.clone() placeholder="Qty" />
                            <input type="text" value=unit.clone() placeholder="Unit" />
                        </div>
                    }.into_view()
                }).collect_view()}
            </div>
            
            <button class="btn btn-secondary" on:click=move |_| {
                // Add item
            }>
                "Add Item"
            </button>
            
            <button class="btn btn-primary" on:click=move |_| {
                // Save stack
            }>
                "Save Stack"
            </button>
        </div>
    }
}

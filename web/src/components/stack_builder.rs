use crate::state::db::{create_stack, search_catalog};
use engine::models::*;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
struct StackFormItem {
    item_id: Uuid,
    name: String,
    quantity: String,
    unit: String,
}

/// Stack creation form. Uses the PERSISTED catalog (localStorage) so that
/// item IDs stored in the stack match what `log_stack` looks up later.
/// `engine::catalog::seed_catalog()` regenerates UUIDs per call — never use
/// it directly for selections that get saved.
#[component]
pub fn StackBuilder(on_created: Callback<String>) -> impl IntoView {
    let stack_name = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());
    let selected_items = RwSignal::new(Vec::<StackFormItem>::new());
    let error = RwSignal::new(None::<String>);

    // Persisted catalog: search_catalog reads (and on first use seeds) localStorage
    let filtered_catalog = move || {
        let q = search_query.get();
        
        search_catalog(&q).unwrap_or_default()
    };

    let add_item = move |item: CatalogItem| {
        selected_items.update(|items| {
            // Avoid duplicates
            if items.iter().any(|i| i.item_id == item.id) {
                return;
            }
            items.push(StackFormItem {
                item_id: item.id,
                name: item.name.clone(),
                quantity: item
                    .dosage_range
                    .as_ref()
                    .map(|d| d.min.to_string())
                    .unwrap_or_else(|| "1".to_string()),
                unit: item
                    .dosage_range
                    .as_ref()
                    .map(|d| d.unit.clone())
                    .unwrap_or_default(),
            });
        });
    };

    let remove_item = move |idx: usize| {
        selected_items.update(|items| {
            items.remove(idx);
        });
    };

    let handle_create = move |_| {
        let name = stack_name.get();
        if name.trim().is_empty() {
            error.set(Some("Stack name is required".to_string()));
            return;
        }
        let items = selected_items.get();
        if items.is_empty() {
            error.set(Some("Add at least one item to the stack".to_string()));
            return;
        }
        error.set(None);

        let stack_items: Vec<StackItem> = items
            .iter()
            .map(|f| StackItem {
                item_id: f.item_id,
                quantity: Some(f.quantity.parse().unwrap_or(1.0)),
                unit: if f.unit.is_empty() {
                    None
                } else {
                    Some(f.unit.clone())
                },
                note: None,
            })
            .collect();

        let stack = Stack {
            id: Uuid::new_v4(),
            user_id: "local-device".to_string(),
            name: name.clone(),
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            items: stack_items,
        };

        match create_stack(&stack) {
            Ok(()) => {
                on_created.run(name);
                stack_name.set(String::new());
                selected_items.set(Vec::new());
            }
            Err(e) => error.set(Some(format!("Failed to create stack: {}", e))),
        }
    };

    view! {
        <div class="stack-builder">
            <h3>"Create New Stack"</h3>
            <div class="form-row">
                <input
                    type="text"
                    placeholder="Stack name"
                    aria-label="Stack name"
                    prop:value=move || stack_name.get()
                    on:input=move |e| { stack_name.set(event_target_value(&e)); }
                />
            </div>

            <div class="form-row">
                <input
                    type="text"
                    placeholder="Search catalog..."
                    aria-label="Search catalog"
                    on:input=move |e| { search_query.set(event_target_value(&e)); }
                />
            </div>

            <div class="catalog-list">
                {move || filtered_catalog().into_iter().take(15).map(|item| {
                    let item_clone = item.clone();
                    let item_name = item.name.clone();
                    let dosage = item.dosage_range.as_ref()
                        .map(|d| format!("{}-{} {}", d.min, d.max, d.unit))
                        .unwrap_or_default();
                    view! {
                        <button
                            type="button"
                            class="catalog-item"
                            on:click=move |_| { add_item(item_clone.clone()) }
                            aria-label=format!("Add {}", item_name)
                        >
                            <span class="catalog-name">{item.name}</span>
                            <span class="catalog-dosage">{dosage}</span>
                        </button>
                    }
                }).collect_view()}
            </div>

            <Show when=move || !selected_items.get().is_empty()>
                <div class="selected-items">
                    <h4>"Selected Items"</h4>
                    {move || {
                        let items = selected_items.get();
                        items.iter().enumerate().map(|(i, item)| {
                            let name = item.name.clone();
                            view! {
                                <div class="selected-item">
                                    <span>{name}</span>
                                    <span>{item.quantity.clone()} {item.unit.clone()}</span>
                                    <button
                                        type="button"
                                        class="remove-item"
                                        on:click=move |_| { remove_item(i) }
                                        aria-label=format!("Remove {}", item.name.clone())
                                    >"X"</button>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </Show>

            <button
                type="button"
                class="save-button"
                on:click=handle_create
                aria-label="Create stack"
            >"Create Stack"</button>

            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">
                    {move || error.get()}
                </div>
            </Show>
        </div>
    }
}

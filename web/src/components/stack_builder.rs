use leptos::*;
use leptos::prelude::*;
use engine::models::*;
use engine::catalog::seed_catalog;
use crate::state::db::create_stack;
use uuid::Uuid;

#[derive(Clone)]
struct StackFormItem {
    item_id: Uuid,
    name: String,
    quantity: String,
    unit: String,
}

#[component]
pub fn StackBuilder(
    on_created: Callback<String>,
) -> impl IntoView {
    let stack_name = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());
    let selected_items = RwSignal::new(Vec::<StackFormItem>::new());

    let catalog = Signal::derive(|| seed_catalog());
    let filtered_catalog = move || {
        let q = search_query.get();
        let items = catalog.get();
        if q.is_empty() {
            items.clone()
        } else {
            items.iter()
                .filter(|item| item.name.to_lowercase().contains(&q.to_lowercase()))
                .cloned()
                .collect()
        }
    };

    let add_item = move |item: CatalogItem| {
        selected_items.update(|items| {
            items.push(StackFormItem {
                item_id: item.id,
                name: item.name.clone(),
                quantity: item.dosage_range.as_ref()
                    .map(|d| d.min.to_string())
                    .unwrap_or_else(|| "1".to_string()),
                unit: item.dosage_range.as_ref()
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
        if name.is_empty() {
            return;
        }

        let items = selected_items.get();
        if items.is_empty() {
            return;
        }

        let stack_items: Vec<StackItem> = items.iter().map(|f| StackItem {
            item_id: f.item_id,
            quantity: Some(f.quantity.parse().unwrap_or(1.0)),
            unit: if f.unit.is_empty() { None } else { Some(f.unit.clone()) },
            note: None,
        }).collect();

        let stack = Stack {
            id: Uuid::new_v4(),
            user_id: "local-device".to_string(),
            name: name.clone(),
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            items: stack_items,
        };

        if let Err(e) = create_stack(&stack) {
            eprintln!("Failed to create stack: {}", e);
        } else {
            on_created.run(name);
            stack_name.set(String::new());
            selected_items.set(Vec::new());
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
                {move || filtered_catalog().into_iter().map(|item| {
                    let item_clone = item.clone();
                    let item_name = item.name.clone();
                    view! {
                        <button
                            type="button"
                            class="catalog-item"
                            on:click=move |_| { add_item(item_clone.clone()) }
                            aria-label=format!("Add {}", item_name)
                        >
                            <span class="catalog-name">{item.name}</span>
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
                            view! {
                                <div class="selected-item">
                                    <span>{item.name.clone()}</span>
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
        </div>
    }
}

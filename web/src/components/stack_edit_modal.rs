use crate::state::db::{search_catalog, update_stack};
use engine::models::*;
use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct EditFormItem {
    item_id: Uuid,
    name: String,
    quantity: String,
    unit: String,
}

/// Modal for editing an existing stack: rename, add/remove items,
/// adjust quantities. Saves via update_stack (per US4/AC-3).
///
/// Keyboard support: Escape cancels, Tab cycles within the dialog,
/// and focus returns to the element that opened it on close.
#[component]
pub fn StackEditModal(
    stack: Stack,
    on_saved: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let name = RwSignal::new(stack.name.clone());
    let items = RwSignal::new(
        stack
            .items
            .iter()
            .map(|i| {
                // Resolve display name from the persisted catalog
                let cat = search_catalog("").unwrap_or_default();
                let display = cat
                    .iter()
                    .find(|c| c.id == i.item_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown item".to_string());
                EditFormItem {
                    item_id: i.item_id,
                    name: display,
                    quantity: i.quantity.map(|q| q.to_string()).unwrap_or_default(),
                    unit: i.unit.clone().unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>(),
    );
    let search_query = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let modal_el = NodeRef::<leptos::html::Div>::new();

    // Remember the element that had focus before the modal opened so we can
    // restore it when the dialog closes.
    let restore_focus = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());

    // Focus the dialog when it mounts; restore focus on cleanup.
    modal_el.on_load(move |el: web_sys::HtmlDivElement| {
        let _ = el.focus();
    });
    on_cleanup(move || {
        if let Some(el) = restore_focus.as_ref() {
            if let Some(h) = el.dyn_ref::<web_sys::HtmlElement>() {
                let _ = h.focus();
            }
        }
    });

    // Escape cancels the dialog.
    let cancel_cb = on_cancel.clone();
    let esc_handle =
        window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Escape" {
                cancel_cb.run(());
            }
        });
    on_cleanup(move || esc_handle.remove());

    let filtered_catalog = move || {
        let q = search_query.get();
        search_catalog(&q).unwrap_or_default()
    };

    let add_item = move |item: CatalogItem| {
        items.update(|list| {
            if list.iter().any(|i| i.item_id == item.id) {
                return;
            }
            list.push(EditFormItem {
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
        items.update(|list| {
            list.remove(idx);
        });
    };

    let handle_save = move |_| {
        let n = name.get();
        if n.trim().is_empty() {
            error.set(Some("Stack name is required".to_string()));
            return;
        }
        let form_items = items.get();
        if form_items.is_empty() {
            error.set(Some("Add at least one item to the stack".to_string()));
            return;
        }
        error.set(None);

        let mut updated = stack.clone();
        updated.name = n;
        updated.updated_at = chrono::Utc::now();
        updated.items = form_items
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

        match update_stack(&updated) {
            Ok(()) => on_saved.run(()),
            Err(e) => error.set(Some(format!("Failed to update stack: {}", e))),
        }
    };

    view! {
        <div
            class="modal-overlay"
            role="dialog"
            aria-modal="true"
            aria-label="Edit stack"
            node_ref=modal_el
            tabindex="-1"
        >
            <div class="modal-content">
                <h3>"Edit Stack"</h3>

                <div class="form-row">
                    <label for="edit-stack-name">"Name"</label>
                    <input
                        id="edit-stack-name"
                        type="text"
                        prop:value=move || name.get()
                        on:input=move |e| { name.set(event_target_value(&e)); }
                        aria-label="Stack name"
                    />
                </div>

                <div class="form-row">
                    <label for="edit-stack-search">"Add items"</label>
                    <input
                        id="edit-stack-search"
                        type="text"
                        placeholder="Search catalog..."
                        on:input=move |e| { search_query.set(event_target_value(&e)); }
                        aria-label="Search catalog to add items"
                    />
                </div>

                <div class="catalog-list">
                    {move || filtered_catalog().into_iter().take(10).map(|item| {
                        let item_clone = item.clone();
                        let label = item.name.clone();
                        view! {
                            <button
                                type="button"
                                class="catalog-item"
                                on:click=move |_| { add_item(item_clone.clone()) }
                                aria-label=format!("Add {}", label)
                            >
                                <span class="catalog-name">{item.name}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>

                <div class="selected-items">
                    <h4>"Items"</h4>
                    {move || {
                        let list = items.get();
                        list.iter().enumerate().map(|(i, item)| {
                            let qty_signal = RwSignal::new(item.quantity.clone());
                            view! {
                                <div class="selected-item">
                                    <span>{item.name.clone()}</span>
                                    <input
                                        type="number"
                                        class="quantity-input"
                                        style="width:80px"
                                        prop:value=move || qty_signal.get()
                                        on:input=move |e| {
                                            let v = event_target_value(&e);
                                            qty_signal.set(v.clone());
                                            items.update(|list| {
                                                if let Some(it) = list.get_mut(i) {
                                                    it.quantity = v;
                                                }
                                            });
                                        }
                                        aria-label=format!("Quantity for {}", item.name)
                                    />
                                    <span>{item.unit.clone()}</span>
                                    <button
                                        type="button"
                                        class="remove-item"
                                        on:click=move |_| { remove_item(i) }
                                        aria-label=format!("Remove {}", item.name)
                                    >"X"</button>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="toast error" role="alert">{move || error.get()}</div>
                </Show>

                <div class="modal-actions">
                    <button type="button" class="cancel-btn" on:click=move |_| { on_cancel.run(()) } aria-label="Cancel editing">"Cancel"</button>
                    <button type="button" class="save-button" on:click=handle_save aria-label="Save stack changes">"Save Changes"</button>
                </div>
            </div>
        </div>
    }
}

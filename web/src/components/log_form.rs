use leptos::*;
use leptos::prelude::*;
use engine::models::CatalogItem;
use engine::models::LogEntry;
use crate::state::db::create_log_entry;

#[component]
pub fn LogForm(
    catalog: Vec<CatalogItem>,
) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_item = RwSignal::new(None::<CatalogItem>);
    let quantity = RwSignal::new(String::new());
    let unit = RwSignal::new(String::new());
    let success = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let show_custom = RwSignal::new(false);
    let custom_name = RwSignal::new(String::new());
    let custom_category = RwSignal::new("Supplement".to_string());

    let filtered_catalog = move || {
        let q = search_query.get();
        if q.is_empty() {
            catalog.clone()
        } else {
            catalog.iter()
                .filter(|item| item.name.to_lowercase().contains(&q.to_lowercase()))
                .cloned()
                .collect()
        }
    };

    let handle_select = move |item: CatalogItem| {
        selected_item.set(Some(item.clone()));
        if let Some(dosage) = &item.dosage_range {
            quantity.set(dosage.min.to_string());
            unit.set(dosage.unit.clone());
        }
    };

    let handle_save = move |_| {
        loading.set(true);
        error.set(None);

        if let Some(item) = selected_item.get() {
            let entry = LogEntry {
                id: uuid::Uuid::new_v4(),
                user_id: "local-device".to_string(),
                item_type: item.category.clone(),
                item_id: None,
                name: item.name.clone(),
                quantity: quantity.get().parse().ok(),
                unit: Some(unit.get().clone()),
                route: None,
                timestamp: chrono::Utc::now(),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            };

            if let Err(e) = create_log_entry(&entry) {
                error.set(Some(format!("Failed to save: {}", e)));
                loading.set(false);
                return;
            }
        
            success.set(true);
            selected_item.set(None);
            quantity.set(String::new());
            unit.set(String::new());
            search_query.set(String::new());
            set_timeout(move || {
                success.set(false);
            }, std::time::Duration::from_millis(2000));
        } else if show_custom.get() && !custom_name.get().is_empty() {
            let cat = custom_category.get();
            let item_type = match cat.as_str() {
                "medication" => engine::models::ItemType::Medication,
                "drug" => engine::models::ItemType::Drug,
                "food" => engine::models::ItemType::Food,
                "action" => engine::models::ItemType::Action,
                _ => engine::models::ItemType::Supplement,
            };
            let entry = LogEntry {
                id: uuid::Uuid::new_v4(),
                user_id: "local-device".to_string(),
                item_type,
                item_id: None,
                name: custom_name.get(),
                quantity: Some(1.0),
                unit: None,
                route: None,
                timestamp: chrono::Utc::now(),
                stack_id: None,
                notes: None,
                acknowledged_interaction: false,
                custom_fields: None,
            };

            if let Err(e) = create_log_entry(&entry) {
                error.set(Some(format!("Failed to save: {}", e)));
            } else {
                success.set(true);
                custom_name.set(String::new());
                show_custom.set(false);
                set_timeout(move || {
                    success.set(false);
                }, std::time::Duration::from_millis(2000));
            }
        } else {
            error.set(Some("Please select or create an item".to_string()));
        }
        loading.set(false);
    };

    view! {
        <div class="log-form">
            <input
                type="text"
                placeholder="Search catalog..."
                on:input=move |e| {
                    search_query.set(event_target_value(&e));
                }
                class="search-input"
                aria-label="Search catalog"
            />

            <div class="catalog-list">
                {move || {
                    filtered_catalog().into_iter().map(|item| {
                        let name = item.name.clone();
                        let dosage_text = item.dosage_range.as_ref()
                            .map(|d| format!("{}–{} {}", d.min, d.max, d.unit))
                            .unwrap_or_else(|| "N/A".to_string());
                        let item_for_select = item.clone();
                        let name_for_closure = name.clone();

                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let selected = name_for_closure.clone();
                                    if selected_item.get().as_ref().map(|s| s.name == selected).unwrap_or(false) {
                                        "catalog-item selected"
                                    } else {
                                        "catalog-item"
                                    }
                                }
                                on:click=move |_| { handle_select(item_for_select.clone()) }
                                aria-label=format!("Select {}", name.clone())
                            >
                                <span class="catalog-name">{name.clone()}</span>
                                <span class="catalog-dosage">{dosage_text}</span>
                            </button>
                        }
                    }).collect_view()
                }}
            </div>

            <button
                type="button"
                class="custom-item-btn"
                on:click=move |_| {
                    show_custom.update(|v| *v = !*v);
                }
            >
                "+ Add Custom Item"
            </button>

            <Show when=move || show_custom.get()>
                <div class="custom-item-form">
                    <input
                        type="text"
                        placeholder="Item name"
                        on:input=move |e| {
                            custom_name.set(event_target_value(&e));
                        }
                        aria-label="Custom item name"
                    />
                    <select
                        on:input=move |e| {
                            custom_category.set(event_target_value(&e));
                        }
                        aria-label="Custom item category"
                    >
                        <option value="Supplement">Supplement</option>
                        <option value="Medication">Medication</option>
                        <option value="Drug">Drug</option>
                        <option value="Food">Food</option>
                        <option value="Action">Action</option>
                    </select>
                    <button
                        type="button"
                        on:click=move |_| {
                            selected_item.set(None);
                            custom_name.set(String::new());
                            show_custom.set(false);
                        }
                    >"Cancel"</button>
                </div>
            </Show>

            <Show when=move || selected_item.get().is_some()>
                <div class="form-section">
                    <h3>"Log: " {move || selected_item.get().map(|i| i.name.clone()).unwrap_or_default()}</h3>
                    <div class="form-row">
                        <label for="quantity">"Quantity:"</label>
                        <input
                            id="quantity"
                            type="number"
                            value=move || quantity.get()
                            on:input=move |e| {
                                quantity.set(event_target_value(&e));
                            }
                            class="quantity-input"
                            aria-label="Quantity"
                        />
                    </div>
                    <div class="form-row">
                        <label for="unit">"Unit:"</label>
                        <input
                            id="unit"
                            type="text"
                            value=move || unit.get()
                            on:input=move |e| {
                                unit.set(event_target_value(&e));
                            }
                            class="unit-input"
                            aria-label="Unit"
                        />
                    </div>
                    <button
                        type="button"
                        class="save-button"
                        disabled=move || loading.get()
                        on:click=handle_save
                        aria-label="Save entry"
                    >
                        {move || if loading.get() { "Saving..." } else { "Save Entry" }}
                    </button>
                </div>
            </Show>

            <Show when=move || success.get()>
                <div class="toast success" role="alert">
                    "Logged successfully!"
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">
                    {move || error.get()}
                </div>
            </Show>
        </div>
    }
}

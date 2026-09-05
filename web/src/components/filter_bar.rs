use leptos::*;
use leptos::prelude::*;

#[derive(Clone, Default)]
pub struct HistoryFilter {
    pub search: String,
    pub category: Option<String>,
}

#[component]
pub fn FilterBar(
    filter: RwSignal<HistoryFilter>,
) -> impl IntoView {
    let categories = vec![
        "All".to_string(),
        "supplement".to_string(),
        "medication".to_string(),
        "drug".to_string(),
        "food".to_string(),
        "action".to_string(),
    ];

    view! {
        <div class="filter-bar">
            <input
                type="text"
                placeholder="Search..."
                prop:value=move || filter.get().search
                on:input=move |e| {
                    filter.update(|f| f.search = event_target_value(&e));
                }
                class="search-input"
                aria-label="Search entries"
            />
            <div class="category-chips">
                {categories.iter().map(|cat| {
                    let cat_for_class = cat.clone();
                    let cat_for_click = cat.clone();
                    let cat_display = match cat.as_str() {
                        "All" => "All",
                        "supplement" => "Supplement",
                        "medication" => "Medication",
                        "drug" => "Drug",
                        "food" => "Food",
                        "action" => "Action",
                        _ => cat,
                    }.to_string();

                    view! {
                        <button
                            type="button"
                            class=move || {
                                let current = filter.get().category.clone();
                                let is_all = cat_for_class == "All";
                                let is_active = if is_all {
                                    current.is_none()
                                } else {
                                    current.as_ref() == Some(&cat_for_class)
                                };
                                if is_active { "chip active" } else { "chip" }
                            }
                            on:click=move |_| {
                                filter.update(|f| {
                                    if cat_for_click == "All" {
                                        f.category = None;
                                    } else if f.category.as_ref() == Some(&cat_for_click) {
                                        f.category = None;
                                    } else {
                                        f.category = Some(cat_for_click.clone());
                                    }
                                });
                            }
                            aria-label=format!("Filter by {}", cat_display.clone())
                        >{cat_display.clone()}</button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
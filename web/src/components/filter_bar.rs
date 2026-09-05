use leptos::*;
use leptos::prelude::*;

#[component]
pub fn FilterBar() -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_category = RwSignal::new(None::<String>);

    let categories = vec![
        "All".to_string(),
        "Supplement".to_string(),
        "Medication".to_string(),
        "Drug".to_string(),
        "Food".to_string(),
        "Action".to_string(),
    ];

    view! {
        <div class="filter-bar">
            <input
                type="text"
                placeholder="Search..."
                on:input=move |e| {
                    search_query.set(event_target_value(&e));
                }
                class="search-input"
                aria-label="Search entries"
            />
            <div class="category-chips">
                {categories.iter().map(|cat| {
                    let cat_for_class = cat.clone();
                    let cat_for_click = cat.clone();
                    let cat_display = cat.clone();

                    view! {
                        <button
                            type="button"
                            class=move || {
                                if selected_category.get() == Some(cat_for_class.clone()) {
                                    "chip active"
                                } else {
                                    "chip"
                                }
                            }
                            on:click=move |_| {
                                selected_category.update(|v| {
                                    *v = if v.as_ref() == Some(&cat_for_click) { None } else { Some(cat_for_click.clone()) };
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

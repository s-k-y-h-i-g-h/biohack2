use engine::models::Stack;
use leptos::prelude::*;
use uuid::Uuid;

/// Displays the user's stacks with Log/Edit/Delete actions. Takes a reactive
/// signal so the list live-updates when stacks are created, edited, or deleted.
#[component]
pub fn StackListView(
    stacks: Signal<Vec<Stack>>,
    on_log: Callback<Uuid>,
    on_delete: Callback<Uuid>,
    on_edit: Option<Callback<Uuid>>,
) -> impl IntoView {
    view! {
        <div class="stack-list">
            {move || {
                let stacks = stacks.get();
                if stacks.is_empty() {
                    view! {
                        <p class="empty-state">"No stacks yet. Create one above!"</p>
                    }.into_any()
                } else {
                    stacks.into_iter().map(|stack| {
                        let stack_id = stack.id;
                        let stack_name = stack.name.clone();
                        let item_count = stack.items.len();
                        let log_click = { let id = stack_id; move |_: leptos::ev::MouseEvent| on_log.run(id) };
                        let delete_click = { let id = stack_id; move |_: leptos::ev::MouseEvent| on_delete.run(id) };
                        let name_for_label = stack_name.clone();

                        let edit_button = on_edit.as_ref().map(|cb| {
                            let id = stack_id;
                            let cb = cb.clone();
                            view! {
                                <button
                                    type="button"
                                    class="edit-stack-btn"
                                    on:click=move |_: leptos::ev::MouseEvent| cb.run(id)
                                    aria-label=format!("Edit stack {}", name_for_label)
                                >"Edit"</button>
                            }
                        });

                        view! {
                            <div class="stack-card">
                                <div class="stack-info">
                                    <span class="stack-name">{stack_name}</span>
                                    <span class="stack-item-count">{item_count} items</span>
                                </div>
                                <div class="stack-actions">
                                    {edit_button}
                                    <button
                                        type="button"
                                        class="log-stack-btn"
                                        on:click=log_click
                                        aria-label=format!("Log stack {}", name_for_label)
                                    >"Log"</button>
                                    <button
                                        type="button"
                                        class="delete-stack-btn"
                                        on:click=delete_click
                                        aria-label=format!("Delete stack {}", name_for_label)
                                    >"Delete"</button>
                                </div>
                            </div>
                        }
                    }).collect_view().into_any()
                }
            }}
        </div>
    }
}

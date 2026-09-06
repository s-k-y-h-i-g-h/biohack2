use leptos::*;
use leptos::prelude::*;
use engine::models::Stack;
use crate::state::db::delete_stack;
use uuid::Uuid;

#[component]
pub fn StackListView(
    stacks: Vec<Stack>,
    on_log: Callback<Uuid>,
    on_delete: Callback<Uuid>,
) -> impl IntoView {
    let has_stacks = !stacks.is_empty();

    view! {
        <div class="stack-list">
            <Show when=move || !has_stacks>
                <p>"No stacks yet. Create one above!"</p>
            </Show>
            <Show when=move || has_stacks>
                {stacks.iter().map(|stack| {
                    let stack_id = stack.id;
                    let stack_name = stack.name.clone();
                    let item_count = stack.items.len();
                    let stack_ref = stack;

                    view! {
                        <div class="stack-card">
                            <div class="stack-info">
                                <span class="stack-name">{stack_name}</span>
                                <span class="stack-item-count">{item_count} items</span>
                            </div>
                            <div class="stack-actions">
                                <button
                                    type="button"
                                    class="log-stack-btn"
                                    on:click=move |_| { on_log.run(stack_id) }
                                    aria-label=format!("Log stack {}", stack_ref.name)
                                >"Log"</button>
                                <button
                                    type="button"
                                    class="delete-stack-btn"
                                    on:click=move |_| { on_delete.run(stack_id) }
                                    aria-label=format!("Delete stack {}", stack_ref.name)
                                >"Delete"</button>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </Show>
        </div>
    }
}

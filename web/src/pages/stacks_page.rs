use leptos::*;
use leptos::prelude::*;
use engine::models::*;
use crate::state::db::{get_stacks, delete_stack, log_stack};
use crate::components::{StackBuilder, StackListView};
use uuid::Uuid;

#[component]
pub fn StacksPage() -> impl IntoView {
    let stacks = RwSignal::new(Vec::<Stack>::new());
    let message = RwSignal::new(None::<String>);

    // Load stacks on mount
    let load_stacks = move || {
        get_stacks().unwrap_or_default()
    };
    stacks.set(load_stacks());

    let handle_created = move |name: String| {
        stacks.set(load_stacks());
        message.set(Some(format!("Stack \"{}\" created!", name)));
        set_timeout(move || {
            message.set(None);
        }, std::time::Duration::from_millis(2000));
    };

    let handle_log = move |stack_id: Uuid| {
        let stack = stacks.get().into_iter()
            .find(|s| s.id == stack_id);

        if let Some(stack) = stack {
            match log_stack(&stack) {
                Ok(_ids) => {
                    message.set(Some(format!("Logged {} items from \"{}\"", stack.items.len(), stack.name)));
                    set_timeout(move || {
                        message.set(None);
                    }, std::time::Duration::from_millis(2000));
                }
                Err(e) => {
                    eprintln!("Failed to log stack: {}", e);
                    message.set(Some(format!("Failed to log stack: {}", e)));
                }
            }
        }
    };

    let handle_delete = move |stack_id: Uuid| {
        if let Err(e) = delete_stack(&stack_id.to_string()) {
            eprintln!("Failed to delete stack: {}", e);
        } else {
            stacks.set(load_stacks());
        }
    };

    view! {
        <div class="page">
            <h2>"Stacks"</h2>

            <Show when=move || message.get().is_some()>
                <div class="toast success">
                    {move || message.get()}
                </div>
            </Show>

            <StackBuilder on_created=Callback::new(handle_created) />

            <StackListView
                stacks=stacks.get_untracked()
                on_log=Callback::new(handle_log)
                on_delete=Callback::new(handle_delete)
            />
        </div>
    }
}

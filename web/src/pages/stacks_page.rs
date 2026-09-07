use leptos::prelude::*;
use engine::models::*;
use crate::state::db::{get_stacks, delete_stack, log_stack};
use crate::state::store::AppContext;
use crate::components::{StackBuilder, StackListView};
use uuid::Uuid;

#[component]
pub fn StacksPage() -> impl IntoView {
    // Global data version — bumps propagate to this page AND Layout's alert banner
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let message = RwSignal::new(None::<String>);

    // Reactive: re-reads stacks from storage whenever version changes
    let stacks = Signal::derive(move || {
        version.get(); // track
        get_stacks().unwrap_or_default()
    });

    let refresh = move || {
        version.update(|v| *v += 1);
    };

    let flash = move |msg: String| {
        message.set(Some(msg));
        set_timeout(
            move || message.set(None),
            std::time::Duration::from_millis(2000),
        );
    };

    let handle_created = move |name: String| {
        refresh();
        flash(format!("Stack \"{}\" created!", name));
    };

    let handle_log = move |stack_id: Uuid| {
        let stack = get_stacks()
            .unwrap_or_default()
            .into_iter()
            .find(|s| s.id == stack_id);

        if let Some(stack) = stack {
            match log_stack(&stack) {
                Ok(ids) => flash(format!(
                    "Logged {} items from \"{}\"",
                    ids.len(),
                    stack.name
                )),
                Err(e) => flash(format!("Failed to log stack: {}", e)),
            }
        }
    };

    let handle_delete = move |stack_id: Uuid| {
        match delete_stack(&stack_id.to_string()) {
            Ok(()) => {
                refresh();
                flash("Stack deleted".to_string());
            }
            Err(e) => flash(format!("Failed to delete stack: {}", e)),
        }
    };

    view! {
        <div class="page">
            <h2>"Stacks"</h2>

            <Show when=move || message.get().is_some()>
                <div class="toast success" role="status">
                    {move || message.get()}
                </div>
            </Show>

            <StackBuilder on_created=Callback::new(handle_created) />

            <StackListView
                stacks=stacks
                on_log=Callback::new(handle_log)
                on_delete=Callback::new(handle_delete)
            />
        </div>
    }
}
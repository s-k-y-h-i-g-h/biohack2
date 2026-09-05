use leptos::*;
use leptos::prelude::*;

#[component]
pub fn OfflineIndicator(
    is_online: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class=move || if is_online.get() { "status-indicator online" } else { "status-indicator offline" }>
            {move || if is_online.get() { "Online" } else { "Offline" }}
        </div>
    }
}

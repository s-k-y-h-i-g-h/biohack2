use leptos::*;
use leptos::prelude::*;

#[component]
pub fn AlertBanner(
    alert: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || alert.get().is_some()>
            <div class="alert-banner warning" role="alert">
                <span class="alert-message">{move || alert.get().clone().unwrap_or_default()}</span>
            </div>
        </Show>
    }
}

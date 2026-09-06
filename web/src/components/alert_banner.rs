use leptos::*;
use leptos::prelude::*;

#[component]
pub fn AlertBanner(
    alert: ReadSignal<Option<String>>,
    on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Show when=move || alert.get().is_some()>
            <div class="alert-banner warning" role="alert">
                <span class="alert-message">{move || alert.get().clone().unwrap_or_default()}</span>
                <button
                    type="button"
                    class="alert-dismiss"
                    on:click=move |_| {
                        if let Some(cb) = &on_dismiss {
                            cb.run(());
                        }
                    }
                    aria-label="Dismiss alert"
                >"×"</button>
            </div>
        </Show>
    }
}

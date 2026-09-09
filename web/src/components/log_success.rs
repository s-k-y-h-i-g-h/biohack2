use leptos::prelude::*;
use leptos::*;

#[component]
pub fn LogSuccess() -> impl IntoView {
    view! {
        <div class="toast success">
            "Logged successfully!"
        </div>
    }
}

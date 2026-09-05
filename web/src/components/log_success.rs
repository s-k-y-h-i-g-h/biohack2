use leptos::*;
use leptos::prelude::*;

#[component]
pub fn LogSuccess() -> impl IntoView {
    view! {
        <div class="toast success">
            "Logged successfully!"
        </div>
    }
}

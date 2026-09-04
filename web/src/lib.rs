use leptos::*;
use leptos::prelude::*;

mod pages;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div style="padding: 20px;">
            <h1>"Biohack Tracker"</h1>
            {pages::log_page()}
        </div>
    }
}

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    })
}

use leptos::*;
use leptos::prelude::*;
use leptos::mount::mount_to_body;
use tachys::html::element::ElementChild;

mod pages;

fn app() -> impl IntoView {
    view! {
        <div style="padding: 20px;">
            <h1>"Biohack Tracker"</h1>
            <nav>
                <a href="/" style="margin-right: 10px;">"Log"</a>
                <a href="/history" style="margin-right: 10px;">"History"</a>
                <a href="/vitals" style="margin-right: 10px;">"Vitals"</a>
                <a href="/stacks">"Stacks"</a>
            </nav>
            {pages::log_page()}
        </div>
    }
}

fn main() {
    mount_to_body(app);
}

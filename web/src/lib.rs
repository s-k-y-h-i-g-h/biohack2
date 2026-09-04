use leptos::*;
use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route};
use leptos_router::path;
use leptos::mount::mount_to_body;

mod pages;
mod state;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div style="padding: 20px; font-family: system-ui, sans-serif;">
                <h1>"Biohack Tracker"</h1>
                <nav style="margin-bottom: 20px;">
                    <a href="/" style="margin-right: 10px; color: #4f8ef7;">"Log"</a>
                    <a href="/history" style="margin-right: 10px; color: #4f8ef7;">"History"</a>
                    <a href="/vitals" style="margin-right: 10px; color: #4f8ef7;">"Vitals"</a>
                    <a href="/stacks" style="color: #4f8ef7;">"Stacks"</a>
                </nav>
                <Routes fallback=|| "Not found">
                    <Route path=path!("") view=pages::log_page::LogPage />
                    <Route path=path!("/history") view=pages::history_page::HistoryPage />
                    <Route path=path!("/vitals") view=pages::vitals_page::VitalsPage />
                    <Route path=path!("/stacks") view=pages::stacks_page::StacksPage />
                </Routes>
            </div>
        </Router>
    }
}

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    })
}

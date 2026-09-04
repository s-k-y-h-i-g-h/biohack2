use leptos::*;
use leptos_router::*;

mod router;
mod app;
mod state;
mod components;
mod pages;
mod styles;

pub use app::App;
pub use state::store;

fn main() {
    leptos::mount_to_body(|| {
        view! {
            <Router>
                <App />
            </Router>
        }
    })
}

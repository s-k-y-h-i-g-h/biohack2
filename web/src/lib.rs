use leptos::*;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

mod pages;
mod components;
pub mod state;
pub mod types;
#[cfg(test)]
mod tests;

use components::Layout;

#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    // Inject global styles by reading from document head
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(head) = doc.head() {
                let css = include_str!("./styles/global.css");
                let style = doc.create_element("style").unwrap();
                style.set_text_content(Some(css));
                let _ = head.append_child(&style);
            }
        }
    }

    mount_to_body(app);
}

fn get_path() -> String {
    if let Some(win) = web_sys::window() {
        // Try pathname first (HTML5 history mode)
        if let Ok(pathname) = win.location().pathname() {
            if !pathname.is_empty() {
                return pathname;
            }
        }
        // Fall back to hash-based routing
        if let Ok(hash) = win.location().hash() {
            if !hash.is_empty() && hash != "#" {
                return hash[1..].to_string();
            }
        }
    }
    "/".to_string()
}

fn app() -> impl IntoView {
    let location = RwSignal::new(get_path());
    let current_path = move || location.get();

    // Listen for popstate (browser back/forward)
    {
        let location = location.clone();
        let listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
            location.set(get_path());
        }) as Box<dyn FnMut(_)>);
        if let Some(win) = web_sys::window() {
            let _ = win
                .add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref());
        }
        listener.forget();
    }

    // Listen for hashchange events
    {
        let location = location.clone();
        let listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
            location.set(get_path());
        }) as Box<dyn FnMut(_)>);
        if let Some(win) = web_sys::window() {
            let _ = win
                .add_event_listener_with_callback("hashchange", listener.as_ref().unchecked_ref());
        }
        listener.forget();
    }

    view! {
        <Layout>
            <main>
                <Show when=move || current_path() == "/history">
                    {pages::history_page()}
                </Show>
                <Show when=move || current_path() == "/vitals">
                    {pages::vitals_page()}
                </Show>
                <Show when=move || current_path() == "/stacks">
                    {pages::stacks_page()}
                </Show>
                <Show when=move || current_path() == "/" || current_path().is_empty()>
                    {pages::LogPage()}
                </Show>
            </main>
        </Layout>
    }
}

use gloo_storage::Storage;
use leptos::prelude::*;
use leptos::*;
use wasm_bindgen::prelude::*;

mod components;
mod pages;
pub mod state;
#[cfg(test)]
mod tests;
pub mod types;

use components::Layout;

#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    // Inject global styles by reading from document head
    if let Some(win) = web_sys::window()
        && let Some(doc) = win.document()
    {
        if let Some(head) = doc.head() {
            let css = include_str!("./styles/global.css");
            let style = doc.create_element("style").unwrap();
            style.set_text_content(Some(css));
            let _ = head.append_child(&style);

            // Inject manifest link
            let link = doc.create_element("link").unwrap();
            let _ = link.set_attribute("rel", "manifest");
            let _ = link.set_attribute("href", "manifest.json");
            let _ = head.append_child(&link);

            // Register service worker (progressive enhancement — not required for the app).
            // Guard with js_sys::Reflect so a browser/context where navigator.serviceWorker
            // is unavailable doesn't throw during init and block the whole app from mounting.
            {
                let navigator = web_sys::js_sys::Reflect::get(&win, &"navigator".into()).unwrap_or(JsValue::UNDEFINED);
                let sw = web_sys::js_sys::Reflect::get(&navigator, &"service_worker".into()).unwrap_or(JsValue::UNDEFINED);
                if !sw.is_undefined()
                    && let Ok(register) = web_sys::js_sys::Reflect::get(&sw, &"register".into())
                    && let Some(reg_fn) = register.dyn_ref::<web_sys::js_sys::Function>()
                {
                    let _ = reg_fn.call1(&sw, &"/sw.js".into());
                }
            }
        }

        // Apply the persisted theme at startup so a saved dark mode
        // survives reloads on every page (was previously applied only
        // when the Settings page happened to be mounted).
        if let Some(body) = doc.body() {
            let saved = gloo_storage::LocalStorage::get::<String>("biohack2_theme")
                .unwrap_or_else(|_| "light".to_string());
            let _ = body.class_list().remove_1("dark");
            let _ = body.class_list().remove_1("light");
            let _ = body.class_list().add_1(&saved);
        }
    }

    mount_to_body(app);

    // Hydrate from the backend: server is the durable source of truth.
    // Runs after mount so the UI renders instantly from the local cache,
    // then refreshes when server state lands (biohack2-sync-complete event).
    crate::state::sync::sync_from_server();
}

fn get_path() -> String {
    if let Some(win) = web_sys::window() {
        // Try hash first (hash-based routing)
        if let Ok(hash) = win.location().hash()
            && !hash.is_empty()
            && hash != "#"
        {
            return hash[1..].to_string();
        }
        // Fall back to pathname (HTML5 history mode)
        if let Ok(pathname) = win.location().pathname()
            && !pathname.is_empty()
        {
            return pathname;
        }
    }
    "/".to_string()
}

fn app() -> impl IntoView {
    // Global shared state: data version signal for cross-page reactivity
    let ctx = crate::state::store::AppContext::new();
    provide_context(ctx);

    let location = RwSignal::new(get_path());
    let current_path = move || location.get();

    // Mirror the route into AppContext so Layout can mark the active nav link
    // (aria-current) without prop-drilling.
    let ctx2 = expect_context::<crate::state::store::AppContext>();
    ctx2.current_path.set(location.get_untracked());
    {
        let current_path_sig = ctx2.current_path;
        Effect::new(move |_| {
            let p = location.get();
            current_path_sig.set(p);
        });
    }

    // Listen for popstate (browser back/forward)
    {
        let listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
            location.set(get_path());
        }) as Box<dyn FnMut(_)>);
        if let Some(win) = web_sys::window() {
            let _ =
                win.add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref());
        }
        listener.forget();
    }

    // Listen for hashchange events
    {
        let listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
            location.set(get_path());
        }) as Box<dyn FnMut(_)>);
        if let Some(win) = web_sys::window() {
            let _ = win
                .add_event_listener_with_callback("hashchange", listener.as_ref().unchecked_ref());
        }
        listener.forget();
    }

    // When the boot sync replaces the local cache from the server, bump the
    // global data version so every reactive reader re-reads storage.
    {
        let ctx_sync = ctx2.clone();
        let listener = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
            ctx_sync.data_version.update(|v| *v += 1);
        }) as Box<dyn FnMut(_)>);
        if let Some(win) = web_sys::window() {
            let _ = win.add_event_listener_with_callback(
                "biohack2-sync-complete",
                listener.as_ref().unchecked_ref(),
            );
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
                <Show when=move || current_path() == "/settings">
                    {pages::settings_page()}
                </Show>
                <Show when=move || current_path() == "/notes">
                    {pages::notes_page()}
                </Show>
                <Show when=move || current_path() == "/" || current_path().is_empty()>
                    {pages::LogPage()}
                </Show>
            </main>
        </Layout>
    }
}

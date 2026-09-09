use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;

const THEME_STORAGE_KEY: &str = "biohack2_theme";

#[component]
pub fn SettingsPage() -> impl IntoView {
    // Load theme from localStorage or default to "light"
    let theme = RwSignal::new(
        LocalStorage::get::<String>(THEME_STORAGE_KEY).unwrap_or_else(|_| "light".to_string()),
    );

    // Apply theme to body when it changes
    let apply_theme = move |new_theme: &str| {
        if let Some(win) = window() {
            if let Some(doc) = win.document() {
                if let Some(body) = doc.body() {
                    let _ = body.class_list().remove_1("dark");
                    let _ = body.class_list().remove_1("light");
                    let _ = body.class_list().add_1(new_theme);
                }
            }
        }
    };

    // Apply on mount
    let _ = create_effect(move |_| {
        apply_theme(&theme.get());
    });

    let toggle_theme = move |_| {
        let current = theme.get();
        let new_theme = if current == "light" {
            "dark".to_string()
        } else {
            "light".to_string()
        };
        let _ = LocalStorage::set(THEME_STORAGE_KEY, &new_theme);
        theme.set(new_theme.clone());
        apply_theme(&new_theme);
    };

    let handle_export = move |_| {
        let result = crate::state::db::export_data();
        match result {
            Ok(csv) => {
                // Create a Blob from the CSV string
                if let Some(win) = window() {
                    if let Some(doc) = win.document() {
                        let array = js_sys::Array::new();
                        array.push(&js_sys::JsString::from(csv));
                        if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array) {
                            // Create object URL
                            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                                // Create and click a download link
                                if let Ok(a) = doc.create_element("a") {
                                    if let Ok(anchor) = a.dyn_into::<web_sys::HtmlElement>() {
                                        let _ = anchor.set_attribute("href", &url);
                                        let _ =
                                            anchor.set_attribute("download", "biohack_export.csv");
                                        let _ = anchor.click();
                                        let _ = web_sys::Url::revoke_object_url(&url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Export failed: {}", e).into());
            }
        }
    };

    view! {
        <div class="page">
            <h2>"Settings"</h2>
            <div class="settings-section">
                <h3>"Theme"</h3>
                <div class="toggle-row">
                    <span>{move || if theme.get() == "light" { "Light" } else { "Dark" }}</span>
                    <button
                        type="button"
                        class="toggle-btn"
                        on:click=toggle_theme
                        aria-label="Toggle theme"
                    >
                        {move || if theme.get() == "light" { "🌙" } else { "☀️" }}
                    </button>
                </div>
            </div>
            <div class="settings-section">
                <h3>"Data"</h3>
                <button
                    type="button"
                    class="export-btn"
                    on:click=handle_export
                    aria-label="Export data as CSV"
                >
                    "Export CSV"
                </button>
                <p class="help-text">"Exports all log entries and vitals as CSV."</p>
            </div>
        </div>
    }
}

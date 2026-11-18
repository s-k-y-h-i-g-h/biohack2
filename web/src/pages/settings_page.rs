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
        if let Some(win) = window()
            && let Some(doc) = win.document()
            && let Some(body) = doc.body()
        {
            let _ = body.class_list().remove_1("dark");
            let _ = body.class_list().remove_1("light");
            let _ = body.class_list().add_1(new_theme);
        }
    };

    // Apply on mount
    let _ = Effect::new(move |_| {
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
                if let Some(win) = window()
                    && let Some(doc) = win.document()
                {
                    let array = js_sys::Array::new();
                    array.push(&js_sys::JsString::from(csv));
                    if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array) {
                        // Create object URL
                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                            // Create and click a download link
                            if let Ok(a) = doc.create_element("a")
                                && let Ok(anchor) = a.dyn_into::<web_sys::HtmlElement>()
                            {
                                let _ = anchor.set_attribute("href", &url);
                                let _ = anchor.set_attribute("download", "biohack_export.csv");
                                anchor.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
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

    // FR-020/FR-021: cloud data lifecycle. Status text doubles as the busy
    // flag — actions are disabled while a request is in flight.
    let cloud_status = RwSignal::new(String::new());
    let cloud_armed = RwSignal::new(false);
    let cloud_busy = move || {
        let s = cloud_status.get();
        s.starts_with("Exporting") || s.starts_with("Deleting")
    };

    let handle_cloud_export = move |_| {
        crate::state::sync::cloud_export(cloud_status);
    };

    let handle_cloud_delete = move |_| {
        if !cloud_armed.get_untracked() {
            cloud_armed.set(true);
            return;
        }
        cloud_armed.set(false);
        crate::state::sync::cloud_delete(cloud_status);
    };

    view! {
        <div class="page">
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
            <div class="settings-section">
                <h3>"Cloud data"</h3>
                <div class="toggle-row">
                    <button
                        type="button"
                        class="export-btn"
                        on:click=handle_cloud_export
                        disabled=cloud_busy
                        aria-label="Export all cloud data as JSON"
                    >
                        "Export cloud JSON"
                    </button>
                    <button
                        type="button"
                        class="export-btn danger"
                        on:click=handle_cloud_delete
                        disabled=cloud_busy
                        aria-label="Delete all cloud data"
                    >
                        {move || if cloud_armed.get() { "Sure?" } else { "Delete cloud data" }}
                    </button>
                </div>
                <p class="help-text">
                    "Export downloads the complete server dataset as JSON — everything needed to
                     run local-only. Deletion erases your account's rows on the server and cannot
                     be undone; local data is untouched."
                </p>
                <p class="help-text" aria-live="polite">{move || cloud_status.get()}</p>
            </div>
        </div>
    }
}

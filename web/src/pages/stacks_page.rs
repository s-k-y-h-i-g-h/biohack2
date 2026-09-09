use crate::components::{StackBuilder, StackEditModal, StackListView};
use crate::state::db::{create_stack, delete_stack, get_stacks, log_stack, search_catalog};
use crate::state::store::AppContext;
use engine::models::*;
use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[component]
pub fn StacksPage() -> impl IntoView {
    // Global data version — bumps propagate to this page AND Layout's alert banner
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let message = RwSignal::new(None::<String>);
    let editing = RwSignal::new(None::<Stack>);

    // Reactive: re-reads stacks from storage whenever version changes
    let stacks = Signal::derive(move || {
        version.get(); // track
        get_stacks().unwrap_or_default()
    });

    let refresh = move || {
        version.update(|v| *v += 1);
    };

    let flash = move |msg: String| {
        message.set(Some(msg));
        set_timeout(
            move || message.set(None),
            std::time::Duration::from_millis(2500),
        );
    };

    let handle_created = move |name: String| {
        refresh();
        flash(format!("Stack \"{}\" created!", name));
    };

    let handle_log = move |stack_id: Uuid| {
        let stack = get_stacks()
            .unwrap_or_default()
            .into_iter()
            .find(|s| s.id == stack_id);

        if let Some(stack) = stack {
            match log_stack(&stack) {
                Ok(ids) => flash(format!(
                    "Logged {} items from \"{}\"",
                    ids.len(),
                    stack.name
                )),
                Err(e) => flash(format!("Failed to log stack: {}", e)),
            }
        }
    };

    let handle_delete = move |stack_id: Uuid| match delete_stack(&stack_id.to_string()) {
        Ok(()) => {
            refresh();
            flash("Stack deleted".to_string());
        }
        Err(e) => flash(format!("Failed to delete stack: {}", e)),
    };

    let open_edit = move |stack_id: Uuid| {
        let stack = get_stacks()
            .unwrap_or_default()
            .into_iter()
            .find(|s| s.id == stack_id);
        if let Some(s) = stack {
            editing.set(Some(s));
        }
    };

    let handle_saved = move |()| {
        editing.set(None);
        refresh();
        flash("Stack updated".to_string());
    };

    let handle_cancel_edit = move |()| {
        editing.set(None);
    };

    // ── YAML export (T047) ────────────────────────────────────────────────────

    let handle_yaml_export = move |_| {
        let stacks = get_stacks().unwrap_or_default();
        let catalog = search_catalog("").unwrap_or_default();

        let mut yaml = String::from("stacks:\n");
        for s in &stacks {
            yaml.push_str(&format!("  - name: \"{}\"\n", s.name.replace('"', "'")));
            yaml.push_str("    items:\n");
            for item in &s.items {
                let name = catalog
                    .iter()
                    .find(|c| c.id == item.item_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| item.item_id.to_string());
                let qty = item.quantity.map(|q| q.to_string()).unwrap_or_default();
                let unit = item.unit.clone().unwrap_or_default();
                yaml.push_str(&format!(
                    "      - item: \"{}\"\n        quantity: {}\n        unit: \"{}\"\n",
                    name.replace('"', "'"),
                    qty,
                    unit
                ));
            }
        }

        if let Some(win) = web_sys::window() {
            if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&js_sys::Array::from_iter([
                js_sys::JsString::from(yaml),
            ])) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                    if let Some(doc) = win.document() {
                        if let Ok(a) = doc.create_element("a") {
                            if let Ok(anchor) = a.dyn_into::<web_sys::HtmlElement>() {
                                let _ = anchor.set_attribute("href", &url);
                                let _ = anchor.set_attribute("download", "stacks.yaml");
                                let _ = anchor.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
                                flash("Exported stacks.yaml".to_string());
                            }
                        }
                    }
                }
            }
        }
    };

    // ── YAML import (T047) ────────────────────────────────────────────────────
    // Reads the selected file and imports stacks; single refresh at the end.

    let handle_file_selected = move |e: leptos::ev::Event| {
        let input = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        let file = input.and_then(|i| i.files()).and_then(|f| f.get(0));
        let Some(file) = file else { return };

        let reader = web_sys::FileReader::new().unwrap();
        let r = reader.clone();

        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let text = r
                .result()
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            match parse_stacks_yaml(&text) {
                Ok(imported) => {
                    let mut count = 0;
                    for s in &imported {
                        if create_stack(s).is_ok() {
                            count += 1;
                        }
                    }
                    version.update(|v| *v += 1);
                    flash(format!("Imported {} stacks from YAML", count));
                }
                Err(err) => flash(format!("YAML import failed: {}", err)),
            }
        })
            as Box<dyn FnMut(web_sys::Event)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        let _ = reader.read_as_text(&file);
    };

    view! {
        <div class="page">
            <h2>"Stacks"</h2>

            <Show when=move || message.get().is_some()>
                <div class="toast success" role="status">
                    {move || message.get()}
                </div>
            </Show>

            <StackBuilder on_created=Callback::new(handle_created) />

            <div class="stack-yaml-actions">
                <button type="button" class="chip" on:click=handle_yaml_export aria-label="Export stacks as YAML">"Export YAML"</button>
                <label class="chip yaml-import-label">
                    "Import YAML"
                    <input
                        type="file"
                        accept=".yaml,.yml,text/yaml"
                        style="display:none"
                        on:change=handle_file_selected
                        aria-label="Import stacks from YAML file"
                    />
                </label>
            </div>

            <StackListView
                stacks=stacks
                on_log=Callback::new(handle_log)
                on_delete=Callback::new(handle_delete)
                on_edit=Some(Callback::new(open_edit))
            />

            <Show when=move || editing.get().is_some()>
                {move || {
                    editing.get().map(|s| {
                        view! {
                            <StackEditModal
                                stack=s
                                on_saved=Callback::new(handle_saved)
                                on_cancel=Callback::new(handle_cancel_edit)
                            />
                        }
                    })
                }}
            </Show>
        </div>
    }
}

/// Minimal YAML subset parser for stack definitions (name + items with
/// quantity/unit). Avoids a full YAML dependency in the WASM bundle.
fn parse_stacks_yaml(text: &str) -> Result<Vec<Stack>, String> {
    let mut stacks = Vec::new();
    let mut current: Option<Stack> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "stacks:" {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- name:") {
            if let Some(s) = current.take() {
                stacks.push(s);
            }
            current = Some(Stack {
                id: Uuid::new_v4(),
                user_id: "local-device".to_string(),
                name: rest.trim().trim_matches('"').trim_matches('\'').to_string(),
                description: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                items: Vec::new(),
            });
        } else if trimmed.starts_with("items:") {
            // item list follows
        } else if let Some(rest) = trimmed.strip_prefix("- item:") {
            if let Some(s) = current.as_mut() {
                // stash the display name in the note field; resolved to catalog ID after parse
                s.items.push(StackItem {
                    item_id: Uuid::nil(),
                    quantity: None,
                    unit: None,
                    note: Some(rest.trim().trim_matches('"').trim_matches('\'').to_string()),
                });
            }
        } else if let Some(rest) = trimmed.strip_prefix("quantity:") {
            if let Some(s) = current.as_mut() {
                if let Some(last) = s.items.last_mut() {
                    last.quantity = rest.trim().parse().ok();
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("unit:") {
            if let Some(s) = current.as_mut() {
                if let Some(last) = s.items.last_mut() {
                    last.unit = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
    }
    if let Some(s) = current.take() {
        stacks.push(s);
    }

    // Resolve item names → catalog IDs
    let catalog = search_catalog("").unwrap_or_default();
    for s in stacks.iter_mut() {
        for item in s.items.iter_mut() {
            if let Some(name) = item.note.take() {
                match catalog
                    .iter()
                    .find(|c| c.name.to_lowercase() == name.to_lowercase())
                {
                    Some(c) => item.item_id = c.id,
                    None => return Err(format!("Unknown item in YAML: \"{}\"", name)),
                }
            }
        }
    }

    Ok(stacks)
}

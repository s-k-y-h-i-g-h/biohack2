use crate::state::db::{delete_target_band, get_target_bands, upsert_target_band};
use crate::state::pk_alerts::refresh_pharmacokinetic_alerts;
use crate::state::store::AppContext;
use engine::models::TargetBand;
use leptos::prelude::*;

/// spec-003 T247 (spec 005 FR-079): target-band editing is a **form**, not
/// an in-panel gesture. Bands drive redose alerts and the dashboard target
/// edges, so every save/delete also regenerates the PK alert generation —
/// same rule the log page follows after a dose save.
#[component]
pub fn TargetBandEditor() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let substance = RwSignal::new(String::new());
    let low = RwSignal::new(String::new());
    let high = RwSignal::new(String::new());
    let window = RwSignal::new("24".to_string());
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);

    let after_write = move || {
        // Alert set depends on which bands exist — regenerate now.
        if let Err(e) = refresh_pharmacokinetic_alerts(chrono::Utc::now()) {
            error.set(Some(format!("Band saved but alert refresh failed — {e}")));
        }
        ctx.data_version.update(|v| *v += 1);
    };

    let handle_save = move |_| {
        error.set(None);
        saved.set(false);
        let parse = |s: String, field: &str| -> Result<f64, String> {
            s.trim()
                .parse::<f64>()
                .map_err(|_| format!("{field} must be a number"))
        };
        let band = TargetBand {
            substance: substance.get(),
            low: match parse(low.get(), "Low") {
                Ok(v) => v,
                Err(e) => {
                    error.set(Some(e));
                    return;
                }
            },
            high: match parse(high.get(), "High") {
                Ok(v) => v,
                Err(e) => {
                    error.set(Some(e));
                    return;
                }
            },
            window_hours: match parse(window.get(), "Window") {
                Ok(v) => v,
                Err(e) => {
                    error.set(Some(e));
                    return;
                }
            },
            updated_at: chrono::Utc::now(),
        };
        // Engine owns the validity rules — don't duplicate them here.
        if let Err(e) = band.validate() {
            error.set(Some(e));
            return;
        }
        if let Err(e) = upsert_target_band(&band) {
            error.set(Some(e));
            return;
        }
        substance.set(String::new());
        low.set(String::new());
        high.set(String::new());
        after_write();
        saved.set(true);
    };

    let handle_delete = move |name: String| {
        error.set(None);
        match delete_target_band(&name) {
            Ok(true) => after_write(),
            Ok(false) => error.set(Some(format!("No band named '{name}'"))),
            Err(e) => error.set(Some(e)),
        }
    };

    let version = ctx.data_version;
    let bands = move || {
        version.get();
        get_target_bands()
    };

    view! {
        <section class="target-band-editor form-section">
            <h3>"Target bands"</h3>
            <p class="dashboard-sub">"Ranges a substance level should stay inside. Drives redose alerts and the dashed edges on each panel."</p>

            <Show when=move || !bands().is_empty() fallback=|| view! {
                <p class="dashboard-empty">"No target bands yet."</p>
            }>
                <table class="band-table">
                    <thead>
                        <tr><th>"Substance"</th><th>"Low"</th><th>"High"</th><th>"Window"</th><th></th></tr>
                    </thead>
                    <tbody>
                        <For
                            each=bands
                            key=|b| b.substance.clone()
                            children=move |b| {
                                let name = b.substance.clone();
                                view! {
                                    <tr>
                                        <td>{b.substance.clone()}</td>
                                        <td>{format!("{:.1}", b.low)}</td>
                                        <td>{format!("{:.1}", b.high)}</td>
                                        <td>{format!("{:.0} h", b.window_hours)}</td>
                                        <td>
                                            <button
                                                type="button"
                                                aria-label=format!("Remove target band for {}", b.substance)
                                                on:click=move |_| handle_delete(name.clone())
                                            >
                                                "Remove"
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </Show>

            <div class="form-row">
                <label for="band_substance">"Substance:"</label>
                <input
                    id="band_substance"
                    type="text"
                    placeholder="Caffeine"
                    prop:value=move || substance.get()
                    on:input=move |e| substance.set(event_target_value(&e))
                />
                <label for="band_low">"Low:"</label>
                <input
                    id="band_low"
                    type="number"
                    step="any"
                    placeholder="50"
                    prop:value=move || low.get()
                    on:input=move |e| low.set(event_target_value(&e))
                />
                <label for="band_high">"High:"</label>
                <input
                    id="band_high"
                    type="number"
                    step="any"
                    placeholder="150"
                    prop:value=move || high.get()
                    on:input=move |e| high.set(event_target_value(&e))
                />
                <label for="band_window">"Window (h):"</label>
                <input
                    id="band_window"
                    type="number"
                    step="any"
                    prop:value=move || window.get()
                    on:input=move |e| window.set(event_target_value(&e))
                />
            </div>
            <button type="button" on:click=handle_save aria-label="Save target band">"Set band"</button>

            <Show when=move || saved.get()>
                <div class="toast success" role="status">"Band saved — alerts refreshed."</div>
            </Show>
            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">{move || error.get()}</div>
            </Show>
        </section>
    }
}

//! Food logging (spec 007, T006).
//!
//! Search the server-side food DB proxy (`/api/food/search`, OpenFoodFacts +
//! USDA), pick a product, enter grams, log it. Nutrients are scaled to the
//! consumed quantity and CACHED in the entry's `custom_fields["nutrition"]`
//! at log time, so history and day totals work offline forever after.
//!
//! The search itself requires the server — unlike the rest of the app there
//! is no local fallback, because the food databases are external by nature.

use crate::state::db::{create_log_entry, get_log_entries};
use crate::state::store::AppContext;
use crate::state::sync::fetch_safe;
use engine::food::{self, DailyNutrition, FoodSearchResult, NutrientInfo};
use engine::models::{ItemType, LogEntry};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Human-readable label for a search result in the log entry name.
fn display_name(r: &FoodSearchResult) -> String {
    match &r.brand {
        Some(b) if !b.trim().is_empty() => format!("{} — {}", r.description, b),
        _ => r.description.clone(),
    }
}

/// Headline nutrients for the today-strip, matched by name prefix against
/// USDA-canonical names (e.g. "Carbohydrate, by difference", "Total lipid (fat)").
fn headline_nutrients(day: &DailyNutrition) -> Vec<NutrientInfo> {
    const HEADLINES: [(&str, &str); 5] = [
        ("Energy", "kcal"),
        ("Protein", "protein"),
        ("Carbohydrate", "carbs"),
        ("Total lipid", "fat"),
        ("Sodium", "sodium"),
    ];
    HEADLINES
        .iter()
        .filter_map(|(prefix, _)| {
            day.totals
                .iter()
                .find(|t| t.name.starts_with(prefix))
                .cloned()
        })
        .collect()
}

/// Compact amount rendering: integers stay integers, else 1 decimal.
fn fmt_amount(n: &NutrientInfo) -> String {
    let v = if n.amount.abs() >= 100.0 {
        n.amount.round()
    } else {
        (n.amount * 10.0).round() / 10.0
    };
    if v.fract() == 0.0 {
        format!("{} {}", v, n.unit)
    } else {
        format!("{:.1} {}", v, n.unit)
    }
}

#[component]
pub fn FoodPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let query = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<FoodSearchResult>::new());
    let searched = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let selected = RwSignal::new(None::<FoodSearchResult>);
    let grams = RwSignal::new("100".to_string());
    let logged = RwSignal::new(None::<String>);

    let run_search = move |_| {
        let q = query.get().trim().to_string();
        if q.is_empty() {
            return;
        }
        loading.set(true);
        error.set(None);
        results.set(Vec::new());
        searched.set(true);
        selected.set(None);
        logged.set(None);
        spawn_local(async move {
            let url = format!("api/food/search?q={}", js_sys::encode_uri_component(&q));
            match fetch_safe("GET", &url, None).await {
                Ok(resp) if resp.ok() => match resp.json::<Vec<FoodSearchResult>>().await {
                    Ok(v) => {
                        results.set(v);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Unreadable food search response: {e:?}")));
                        loading.set(false);
                    }
                },
                Ok(resp) => {
                    error.set(Some(format!(
                        "Food search failed (HTTP {}) — is the biohack2 server running?",
                        resp.status()
                    )));
                    loading.set(false);
                }
                Err(_) => {
                    error.set(Some(
                        "Food search unreachable — the server backend is offline. \
                         Logged foods stay available offline; only lookup needs the server."
                            .to_string(),
                    ));
                    loading.set(false);
                }
            }
        });
    };

    let handle_log = move |_| {
        let Some(sel) = selected.get() else {
            return;
        };
        let g: f64 = grams.get().trim().parse().unwrap_or(0.0);
        if g <= 0.0 || !g.is_finite() {
            error.set(Some("Enter a positive amount in grams".to_string()));
            return;
        }
        error.set(None);
        let scaled = sel
            .nutrients_per_100g
            .as_deref()
            .map(|n| food::scale_per_100g(n, g))
            .unwrap_or_default();
        let name = display_name(&sel);
        let mut entry = LogEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "local-device".to_string(),
            item_type: ItemType::Food,
            item_id: None,
            name: name.clone(),
            quantity: Some(g),
            unit: Some("g".to_string()),
            route: None,
            timestamp: chrono::Utc::now(),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        if !scaled.is_empty() {
            food::set_nutrition(&mut entry, &scaled);
        }
        match create_log_entry(&entry) {
            Ok(()) => {
                logged.set(Some(name));
                selected.set(None);
                version.update(|v| *v += 1);
            }
            Err(e) => error.set(Some(format!("Failed to save food entry: {e}"))),
        }
    };

    // Today's totals, recomputed on every data write (incl. server sync).
    let today = move || {
        version.get(); // track
        let entries = get_log_entries().unwrap_or_default();
        food::aggregate_day(&entries, chrono::Utc::now().date_naive())
    };

    // Live preview of what the current grams will store.
    let preview = move || {
        let sel = selected.get()?;
        let per100 = sel.nutrients_per_100g.as_deref()?;
        let g: f64 = grams.get().trim().parse().unwrap_or(0.0);
        if g <= 0.0 {
            return Some(Vec::new());
        }
        Some(food::scale_per_100g(per100, g))
    };

    view! {
        <div class="food-page">
            <h2>"Food"</h2>

            <div class="food-today" aria-live="polite">
                <h3>"Today"</h3>
                {move || {
                    let day = today();
                    let heads = headline_nutrients(&day);
                    view! {
                        <div class="food-totals">
                            {if day.entry_count == 0 && day.entries_without_nutrition == 0 {
                                view! { <span class="food-empty">"Nothing logged yet today"</span> }.into_any()
                            } else {
                                view! {
                                    {heads.into_iter().map(|n| view! {
                                        <span class="food-total-chip">{fmt_amount(&n)}</span>
                                    }).collect_view()}
                                    <span class="food-entry-count">
                                        {format!("{} food entries", day.entry_count)}
                                    </span>
                                    <Show when=move || day.entries_without_nutrition != 0>
                                        <span class="food-incomplete">
                                            {format!(
                                                "{} without nutrition data",
                                                day.entries_without_nutrition
                                            )}
                                        </span>
                                    </Show>
                                }
                                .into_any()
                            }}
                        </div>
                    }
                }}
            </div>

            <form
                class="food-search"
                on:submit=move |ev| {
                    ev.prevent_default();
                    run_search(());
                }
            >
                <input
                    type="search"
                    class="search-input"
                    placeholder="Search foods (OpenFoodFacts + USDA)..."
                    aria-label="Search foods"
                    prop:value=move || query.get()
                    on:input=move |e| query.set(event_target_value(&e))
                />
                <button type="submit" class="save-button" disabled=move || loading.get()>
                    {move || if loading.get() { "Searching..." } else { "Search" }}
                </button>
            </form>

            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">{move || error.get()}</div>
            </Show>

            <Show when=move || logged.get().is_some()>
                <div class="toast success" role="status">
                    {move || format!("Logged {}", logged.get().unwrap_or_default())}
                </div>
            </Show>

            <Show when=move || searched.get() && !loading.get()>
                <div class="food-results">
                    {move || {
                        let list = results.get();
                        if list.is_empty() {
                            return view! {
                                <p class="food-empty">"No foods found — try a simpler name or a barcode."</p>
                            }
                            .into_any();
                        }
                        list.into_iter()
                            .map(|r| {
                                let rid = r.id.clone();
                                let label = display_name(&r);
                                let source = r.source.label().to_string();
                                let has_data = r.nutrients_per_100g.is_some();
                                let per100_kcal = r
                                    .nutrients_per_100g
                                    .as_deref()
                                    .and_then(|n| n.iter().find(|x| x.name == "Energy").cloned())
                                    .map(|e| format!(" · {} /100g", fmt_amount(&e)));
                                view! {
                                    <button
                                        type="button"
                                        class=move || if selected.get().map(|s| s.id == rid).unwrap_or(false) {
                                            "food-result selected"
                                        } else {
                                            "food-result"
                                        }
                                        aria-label=format!("Log {}", label)
                                        on:click=move |_| {
                                            selected.set(Some(r.clone()));
                                            error.set(None);
                                        }
                                    >
                                        <span class="food-result-name">{label.clone()}</span>
                                        <span class="food-result-meta">
                                            {format!(
                                                "{}{}{}",
                                                source,
                                                per100_kcal.unwrap_or_default(),
                                                if has_data { "" } else { " · no nutrition data" }
                                            )}
                                        </span>
                                    </button>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </div>
            </Show>

            <Show when=move || selected.get().is_some()>
                <div class="form-section">
                    <h3>{move || format!("Log: {}", selected.get().map(|s| display_name(&s)).unwrap_or_default())}</h3>
                    <div class="form-row">
                        <label for="food-grams">"Grams:"</label>
                        <input
                            id="food-grams"
                            type="number"
                            min="1"
                            step="1"
                            class="quantity-input"
                            aria-label="Amount in grams"
                            prop:value=move || grams.get()
                            on:input=move |e| grams.set(event_target_value(&e))
                        />
                    </div>
                    <div class="food-preview" aria-live="polite">
                        {move || match preview() {
                            None => view! {
                                <span class="food-empty">"No cached nutrition — logs as a plain food entry"</span>
                            }
                            .into_any(),
                            Some(list) if list.is_empty() => view! {
                                <span class="food-empty">"Enter grams to preview nutrition"</span>
                            }
                            .into_any(),
                            Some(list) => view! {
                                <div class="food-preview-grid">
                                    {list.iter().take(8).map(|n| view! {
                                        <span class="food-preview-item">{format!("{}: {}", n.name, fmt_amount(n))}</span>
                                    }).collect_view()}
                                </div>
                            }
                            .into_any(),
                        }}
                    </div>
                    <button type="button" class="save-button" on:click=handle_log aria-label="Log this food">
                        "Log Food"
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::food::FoodSource;

    fn day_with(totals: Vec<(&str, f64, &str)>) -> DailyNutrition {
        DailyNutrition {
            date: chrono::NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
            entry_count: totals.len(),
            entries_without_nutrition: 0,
            totals: totals
                .into_iter()
                .map(|(name, amount, unit)| NutrientInfo {
                    name: name.into(),
                    amount,
                    unit: unit.into(),
                })
                .collect(),
        }
    }

    #[test]
    fn display_name_includes_brand_when_present() {
        let r = FoodSearchResult {
            id: "1".into(),
            description: "Greek yogurt".into(),
            brand: Some("Fage".into()),
            source: FoodSource::OpenFoodFacts,
            nutrients_per_100g: None,
        };
        assert_eq!(display_name(&r), "Greek yogurt — Fage");
        let r2 = FoodSearchResult { brand: None, ..r };
        assert_eq!(display_name(&r2), "Greek yogurt");
    }

    #[test]
    fn headlines_match_canonical_names_by_prefix() {
        let day = day_with(vec![
            ("Energy", 2500.0, "kcal"),
            ("Protein", 120.4, "g"),
            ("Carbohydrate, by difference", 250.0, "g"),
            ("Total lipid (fat)", 80.0, "g"),
            ("Sodium", 1900.0, "mg"),
            ("Sugars, total", 40.0, "g"), // not a headline
        ]);
        let h = headline_nutrients(&day);
        assert_eq!(h.len(), 5);
        assert!(h.iter().all(|n| n.name != "Sugars, total"));
    }

    #[test]
    fn headlines_skip_missing_without_inventing() {
        let day = day_with(vec![("Protein", 60.0, "g")]);
        let h = headline_nutrients(&day);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].name, "Protein");
    }

    #[test]
    fn fmt_amount_rounds_sensibly() {
        assert_eq!(
            fmt_amount(&NutrientInfo {
                name: "Energy".into(),
                amount: 2500.4,
                unit: "kcal".into()
            }),
            "2500 kcal"
        );
        assert_eq!(
            fmt_amount(&NutrientInfo {
                name: "Protein".into(),
                amount: 12.44,
                unit: "g".into()
            }),
            "12.4 g"
        );
        assert_eq!(
            fmt_amount(&NutrientInfo {
                name: "Sodium".into(),
                amount: 99.95,
                unit: "mg".into()
            }),
            "100 mg"
        );
    }
}

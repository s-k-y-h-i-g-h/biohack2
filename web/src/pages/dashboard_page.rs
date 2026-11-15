use crate::components::{ActiveDosePanel, TargetBandEditor};
use crate::state::db::{get_log_entries, get_target_bands, search_catalog};
use crate::state::store::AppContext;
use engine::dashboard::{PanelView, panel_view};
use engine::pk_assignments::build_models;
use engine::pk_params::SubstanceModel;
use leptos::prelude::*;

/// spec-003 T246 (spec 005 Phase 3): the dashboard page — one
/// ActiveDosePanel per substance the user actually deals with (has a target
/// band or a log entry), derived from stored data on every write.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    let panels = move || {
        // Recompute on every data change; `now` is the render moment.
        version.get();
        build_panels().unwrap_or_else(|e| {
            // Storage failures must not blank the page silently.
            vec![PanelView {
                substance: "Dashboard".into(),
                model_class: engine::models::ModelClass::None,
                series: Vec::new(),
                now_index: 0,
                now_reading: None,
                target: None,
                below_target: false,
                band_is_assumption: false,
                note: format!("Could not read local data — {e}"),
            }]
        })
    };

    view! {
        <div class="dashboard-page">
            <h2>"Active doses"</h2>
            <p class="dashboard-sub">
                "Modelled levels for substances you track — shaded band is the low/high range, dashed lines are target edges."
            </p>
            <Show
                when=move || !panels().is_empty()
                fallback=|| view! {
                    <div class="dashboard-empty">
                        "Nothing tracked yet. Log a dose or set a target band and its panel appears here."
                    </div>
                }
            >
                <div class="dashboard-grid">
                    <For
                        each=panels
                        key=|p| p.substance.to_lowercase()
                        children=move |view| view! { <ActiveDosePanel view=view/> }
                    />
                </div>
            </Show>
            <TargetBandEditor/>
        </div>
    }
}

/// One panel per tracked substance. Substances without a catalog model get
/// an explicit no-model panel (T239 rule: show the gap, never a zero line).
fn build_panels() -> Result<Vec<PanelView>, String> {
    let entries = get_log_entries()?;
    let bands = get_target_bands();
    let catalog = search_catalog("")?;
    let models = build_models(&catalog);
    let now = chrono::Utc::now();
    let key = |s: &str| s.trim().to_lowercase();

    // Ordered wanted list: target bands first (user's explicit intent),
    // then logged substances; deduped case-insensitively.
    let mut wanted: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for name in bands
        .iter()
        .map(|b| b.substance.clone())
        .chain(entries.iter().map(|e| e.name.clone()))
    {
        if seen.insert(key(&name)) {
            wanted.push(name);
        }
    }

    let mut views = Vec::new();
    for name in wanted {
        let model = models
            .iter()
            .find(|m| key(&m.substance) == key(&name))
            .cloned()
            .unwrap_or_else(|| {
                SubstanceModel::unmodelled(name.clone(), "not in the shipped catalog", 0)
            });
        let target = bands.iter().find(|b| key(&b.substance) == key(&name));
        // panel_view only fails on sampling math; one bad substance must
        // not blank the grid, so a failed panel becomes a note panel.
        let view = panel_view(&model, &entries, target, now).unwrap_or_else(|e| PanelView {
            substance: name.clone(),
            model_class: model.model_class,
            series: Vec::new(),
            now_index: 0,
            now_reading: None,
            target: target.cloned(),
            below_target: false,
            band_is_assumption: false,
            note: format!("Could not compute — {e}"),
        });
        views.push(view);
    }
    Ok(views)
}

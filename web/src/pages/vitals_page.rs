use crate::components::{VitalsDashboard, VitalsForm};
use crate::state::db::{
    create_alert, create_vitals_entry, get_alerts, get_log_entries, get_vitals_entries,
};
use crate::state::store::AppContext;
use engine::models::*;
use engine::safety::{RecentSubstance, SafetyEngine};
use leptos::prelude::*;

#[component]
pub fn VitalsPage() -> impl IntoView {
    // Global data version — bumps update this page AND the Layout banner
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    // Reactive: re-reads vitals from storage whenever version changes
    let recent_vitals = Signal::derive(move || {
        version.get(); // track
        get_vitals_entries(&Default::default()).unwrap_or_default()
    });

    // Reactive: re-reads unacknowledged alerts whenever version changes.
    // Kept for future page-level alert UI; the visible banner lives in Layout.
    #[allow(unused_variables)]
    let unacknowledged_alerts = Signal::derive(move || {
        version.get(); // track
        get_alerts(&AlertFilter {
            user_id: Some("local-device".to_string()),
            acknowledged: Some(false),
        })
        .unwrap_or_default()
    });

    let refresh = move || {
        version.update(|v| *v += 1);
    };

    let handle_save = move |entry: VitalsEntry| {
        if let Err(e) = create_vitals_entry(&entry) {
            web_sys::console::error_1(&format!("Failed to save vitals: {}", e).into());
            return;
        }

        // Run safety checks with recent log context
        let engine = SafetyEngine::new();
        let recent_logs = get_log_entries().unwrap_or_default();
        let substances = RecentSubstance::from_log_entries(&recent_logs);

        let safety_result = engine.check_vitals(&entry, &substances);

        // Persist any triggered alerts, enriched with log-derived contextual advice (FR-009)
        for mut alert in safety_result.alerts {
            if let Some(advice) = engine::safety::contextual_advice(&alert, &substances) {
                let rec = alert.recommendation.clone().unwrap_or_default();
                alert.recommendation = Some(format!("{} Context: {}", rec, advice));
            }
            if let Err(e) = create_alert(&alert) {
                web_sys::console::error_1(&format!("Failed to create alert: {}", e).into());
            }
        }

        // Bump global version so this page AND Layout's banner re-read storage
        refresh();
    };

    view! {
        <div class="page">
            <h2>"Vitals"</h2>
            <div class="vitals-container">
                <div class="vitals-form-section">
                    <VitalsForm on_save=Callback::new(handle_save) />
                </div>
                <div class="vitals-dashboard-section">
                    <VitalsDashboard recent_vitals=recent_vitals />
                </div>
            </div>
        </div>
    }
}

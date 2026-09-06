use leptos::*;
use leptos::prelude::*;
use engine::models::*;
use engine::safety::{SafetyEngine, RecentSubstance};
use crate::state::db::{create_vitals_entry, create_alert, get_vitals_entries, get_log_entries, acknowledge_alert, get_alerts};
use crate::components::{VitalsForm, VitalsDashboard, AlertBanner};

#[component]
pub fn VitalsPage() -> impl IntoView {
    let alerts = get_alerts(&AlertFilter { user_id: Some("local-device".to_string()), acknowledged: Some(false) }).unwrap_or_default();

    let current_alert = RwSignal::new(None::<String>);

    if let Some(first_alert) = alerts.first() {
        current_alert.set(Some(first_alert.message.clone()));
    }

    let recent_vitals = Signal::derive(move || {
        get_vitals_entries(&Default::default()).unwrap_or_default()
    });

    let handle_save = move |entry: VitalsEntry| {
        // Save the vitals entry
        if let Err(e) = create_vitals_entry(&entry) {
            eprintln!("Failed to save vitals: {}", e);
            return;
        }

        // Run safety checks
        let engine = SafetyEngine::new();
        let recent_logs = get_log_entries().unwrap_or_default();
        let substances: Vec<RecentSubstance> = recent_logs.iter().map(|log| RecentSubstance {
            name: log.name.clone(),
            category: log.item_type.clone(),
            taken_at: log.timestamp,
            is_stimulant: false, // Would need catalog lookup for this
            is_serotonergic: false,
        }).collect();

        let safety_result = engine.check_vitals(&entry, &substances);

        // Create alerts for any triggered protocols
        for alert in &safety_result.alerts {
            if let Err(e) = create_alert(alert) {
                eprintln!("Failed to create alert: {}", e);
            }
        }

        // Update the alert signal if there are new unacknowledged alerts
        if !safety_result.alerts.is_empty() {
            current_alert.set(Some(safety_result.alerts[0].message.clone()));
        }
    };

    view! {
        <div class="page">
            <h2>"Vitals"</h2>
            <AlertBanner
                alert=current_alert.read_only()
                on_dismiss=Some(Callback::new(move |_| {
                    // Clear the current alert
                    current_alert.set(None);
                }))
            />
            <div class="vitals-container">
                <div class="vitals-form-section">
                    <VitalsForm on_save=Callback::new(handle_save) />
                </div>
                <div class="vitals-dashboard-section">
                    <VitalsDashboard recent_vitals=recent_vitals.get() />
                </div>
            </div>
        </div>
    }
}

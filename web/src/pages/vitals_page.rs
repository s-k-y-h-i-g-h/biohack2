use leptos::*;
use leptos::prelude::*;
use engine::models::*;
use engine::safety::{SafetyEngine, RecentSubstance};
use crate::state::db::{create_vitals_entry, create_alert, get_vitals_entries, get_log_entries, acknowledge_alert, get_alerts};
use crate::components::{VitalsForm, VitalsDashboard, AlertBanner};

#[component]
pub fn VitalsPage() -> impl IntoView {
    let unacknowledged_alerts = RwSignal::new(Vec::<Alert>::new());

    // Load current unacknowledged alerts
    let load_alerts = move || {
        get_alerts(&AlertFilter {
            user_id: Some("local-device".to_string()),
            acknowledged: Some(false)
        }).unwrap_or_default()
    };
    unacknowledged_alerts.set(load_alerts());

    // Refresh alerts after dismissal
    let refresh_alerts = move || {
        unacknowledged_alerts.set(load_alerts());
    };

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
            is_stimulant: false,
            is_serotonergic: false,
        }).collect();

        let safety_result = engine.check_vitals(&entry, &substances);

        // Create alerts for any triggered protocols
        for alert in &safety_result.alerts {
            if let Err(e) = create_alert(alert) {
                eprintln!("Failed to create alert: {}", e);
            }
        }

        // Refresh alerts list
        refresh_alerts();
    };

    // Create a signal that maps Vec<Alert> to Option<String> for the alert banner
    let alert_message = Signal::derive(move || {
        unacknowledged_alerts.get_untracked().first().map(|a| a.message.clone())
    });

    view! {
        <div class="page">
            <h2>"Vitals"</h2>
            <AlertBanner
                alert=alert_message
                on_dismiss=Some(Callback::new(move |_| {
                    // Clear the alert and acknowledge in DB
                    if let Some(alert) = unacknowledged_alerts.get_untracked().first() {
                        let _ = acknowledge_alert(&alert.id);
                    }
                    refresh_alerts();
                }))
            />
            <div class="vitals-container">
                <div class="vitals-form-section">
                    <VitalsForm on_save=Callback::new(handle_save) />
                </div>
                <div class="vitals-dashboard-section">
                    <VitalsDashboard recent_vitals=recent_vitals.get_untracked() />
                </div>
            </div>
        </div>
    }
}
use leptos::*;
use leptos::prelude::*;
use crate::components::{VitalsForm, VitalsDashboard, AlertBanner};
use crate::state::db::{get_vitals_entries, get_alerts};
use engine::models::AlertFilter;

#[component]
pub fn VitalsPage() -> impl IntoView {
    let alerts = get_alerts(&AlertFilter { user_id: Some("local-device".to_string()), acknowledged: Some(false) }).unwrap_or_default();
    
    let current_alert = RwSignal::new(None::<String>);
    
    if let Some(first_alert) = alerts.first() {
        current_alert.set(Some(first_alert.message.clone()));
    }
    
    view! {
        <div class="page">
            <h2>"Vitals"</h2>
            <AlertBanner alert=current_alert.read_only() />
            <div class="vitals-container">
                <div class="vitals-form-section">
                    <VitalsForm on_save=Callback::new(|_: String| {}) />
                </div>
                <div class="vitals-dashboard-section">
                    <VitalsDashboard recent_vitals=vec![] />
                </div>
            </div>
        </div>
    }
}

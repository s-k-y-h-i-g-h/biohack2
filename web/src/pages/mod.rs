use leptos::*;
use leptos::prelude::*;
use engine::models::CatalogItem;
use engine::catalog::seed_catalog;
use crate::components::LogForm;

pub fn log_page() -> impl IntoView {
    let catalog = seed_catalog();
    
    view! {
        <div class="page">
            <h2>"Log Consumption"</h2>
            <LogForm catalog=catalog />
        </div>
    }
}

pub fn history_page() -> impl IntoView {
    let entries = crate::state::db::get_log_entries().unwrap_or_default();
    
    view! {
        <div class="page">
            <h2>"History"</h2>
            <crate::components::FilterBar />
            <div class="history-container">
                <div class="history-list">
                    <crate::components::HistoryView entries=entries.clone() />
                </div>
                <div class="history-sidebar">
                    <crate::components::SummaryStats entries=entries />
                </div>
            </div>
        </div>
    }
}

pub fn vitals_page() -> impl IntoView {
    let alerts = crate::state::db::get_alerts(&engine::models::AlertFilter { user_id: Some("local-device".to_string()), acknowledged: Some(false) }).unwrap_or_default();
    
    let current_alert = RwSignal::new(None::<String>);
    
    if let Some(first_alert) = alerts.first() {
        current_alert.set(Some(first_alert.message.clone()));
    }
    
    view! {
        <div class="page">
            <h2>"Vitals"</h2>
            <crate::components::AlertBanner alert=current_alert.read_only() />
            <div class="vitals-container">
                <div class="vitals-form-section">
                    <crate::components::VitalsForm on_save=Callback::new(|_: String| {}) />
                </div>
                <div class="vitals-dashboard-section">
                    <crate::components::VitalsDashboard recent_vitals=vec![] />
                </div>
            </div>
        </div>
    }
}

pub fn stacks_page() -> impl IntoView {
    view! {
        <div class="page">
            <h2>"Stacks"</h2>
            <p>"Create and manage your supplement stacks."</p>
            <div>"Stack management coming soon..."</div>
        </div>
    }
}

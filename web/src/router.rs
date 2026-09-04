use leptos::*;
use leptos_router::*;

use crate::pages::{log_page::LogPage, history_page::HistoryPage, vitals_page::VitalsPage};
use crate::pages::{stacks_page::StacksPage, insights_page::InsightsPage, settings_page::SettingsPage};

#[component]
pub fn RouterView() -> impl IntoView {
    view! {
        <Routes>
            <Route path="/" view=LogPage />
            <Route path="/log" view=LogPage />
            <Route path="/history" view=HistoryPage />
            <Route path="/vitals" view=VitalsPage />
            <Route path="/stacks" view=StacksPage />
            <Route path="/insights" view=InsightsPage />
            <Route path="/settings" view=SettingsPage />
        </Routes>
    }
}

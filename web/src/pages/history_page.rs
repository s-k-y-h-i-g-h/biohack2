use leptos::*;
use leptos::prelude::*;
use crate::components::{HistoryView, FilterBar, SummaryStats};
use crate::state::db::get_log_entries;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let entries = get_log_entries().unwrap_or_default();
    
    view! {
        <div class="page">
            <h2>"History"</h2>
            <FilterBar />
            <div class="history-container">
                <div class="history-list">
                    <HistoryView entries=entries.clone() />
                </div>
                <div class="history-sidebar">
                    <SummaryStats entries=entries />
                </div>
            </div>
        </div>
    }
}

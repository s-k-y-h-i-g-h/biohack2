use leptos::*;

#[component]
pub fn SummaryStats() -> impl IntoView {
    view! {
        <div class="summary-stats">
            <h2>"Summary"</h2>
            <div class="stats-grid">
                <div class="stat-card">
                    <span class="stat-value">"0"</span>
                    <span class="stat-label">"Total Entries"</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value">"0"</span>
                    <span class="stat-label">"This Week"</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value">"0"</span>
                    <span class="stat-label">"Unique Items"</span>
                </div>
            </div>
        </div>
    }
}

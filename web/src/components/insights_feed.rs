use leptos::*;

#[component]
pub fn InsightsFeed() -> impl IntoView {
    view! {
        <div class="insights-feed">
            <h2>"Insights"</h2>
            <div class="empty-state">
                <p>"Log data for at least 7 days to see insights."</p>
            </div>
        </div>
    }
}

use leptos::*;
use leptos::prelude::*;
use engine::models::Alert;
use engine::models::AlertSeverity;

#[component]
pub fn InteractionWarning(
    alert: Alert,
    on_acknowledge: Callback<()>,
) -> impl IntoView {
    let severity_class = match alert.severity {
        AlertSeverity::Critical => "critical",
        AlertSeverity::Warning => "warning",
        AlertSeverity::Info => "info",
    };
    let message = alert.message.clone();
    let recommendation = alert.recommendation.clone();
    let has_recommendation = recommendation.is_some();
    let rec_text = recommendation.unwrap_or_default();

    view! {
        <div class=format!("interaction-warning {}", severity_class) role="alert">
            <div class="warning-header">
                <span class="warning-icon">[!]</span>
                <span class="warning-title">{message}</span>
            </div>
            <Show when=move || has_recommendation>
                <div class="warning-recommendation">
                    {rec_text.clone()}
                </div>
            </Show>
            <button
                type="button"
                class="acknowledge-btn"
                on:click=move |_| { on_acknowledge.run(()) }
                aria-label="Acknowledge warning"
            >"I understand"</button>
        </div>
    }
}

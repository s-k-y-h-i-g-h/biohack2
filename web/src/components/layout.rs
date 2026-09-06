use leptos::*;
use leptos::prelude::*;
use crate::state::db::{get_alerts, acknowledge_alert};
use crate::components::AlertBanner;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_online = RwSignal::new(true);

    // Load unacknowledged alerts
    let alerts = RwSignal::new(Vec::<engine::models::Alert>::new());
    let load_alerts = move || {
        get_alerts(&engine::models::AlertFilter {
            user_id: Some("local-device".to_string()),
            acknowledged: Some(false)
        }).unwrap_or_default()
    };
    alerts.set(load_alerts());

    let refresh_alerts = move || {
        alerts.set(load_alerts());
    };

    view! {
        <div class="app-container">
            <h1>"Biohack Tracker"</h1>
            <nav class="nav">
                <a href="/" class="nav-link" aria-label="Go to Log page">"Log"</a>
                <a href="/history" class="nav-link" aria-label="Go to History page">"History"</a>
                <a href="/vitals" class="nav-link" aria-label="Go to Vitals page">"Vitals"</a>
                <a href="/stacks" class="nav-link" aria-label="Go to Stacks page">"Stacks"</a>
                <Show when=move || !is_online.get()>
                    <span class="offline-indicator" aria-live="polite">"Offline"</span>
                </Show>
            </nav>
            <AlertBanner
                alert=Signal::derive(move || {
                    alerts.get().first().map(|a| a.message.clone())
                })
                on_dismiss=Some(Callback::new(move |_| {
                    if let Some(alert) = alerts.get_untracked().first() {
                        let _ = acknowledge_alert(&alert.id);
                    }
                    refresh_alerts();
                }))
            />
            <main>
                {children()}
            </main>
        </div>
    }
}

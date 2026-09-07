use leptos::prelude::*;
use crate::state::db::{get_alerts, acknowledge_alert};
use crate::state::store::AppContext;
use crate::components::AlertBanner;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_online = RwSignal::new(true);

    // Global data version — bumped by pages after writes (see AppContext)
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    // Banner reads alerts reactively: updates on every data write, not just at mount
    let alert_message = Signal::derive(move || {
        version.get(); // track
        get_alerts(&engine::models::AlertFilter {
            user_id: Some("local-device".to_string()),
            acknowledged: Some(false),
        })
        .unwrap_or_default()
        .first()
        .map(|a| a.message.clone())
    });

    view! {
        <div class="app-container">
            <h1>"Biohack Tracker"</h1>
            <nav class="nav">
                <a href="#/" class="nav-link" aria-label="Go to Log page">"Log"</a>
                <a href="#/history" class="nav-link" aria-label="Go to History page">"History"</a>
                <a href="#/vitals" class="nav-link" aria-label="Go to Vitals page">"Vitals"</a>
                <a href="#/notes" class="nav-link" aria-label="Go to Notes page">"Notes"</a>
                <a href="#/stacks" class="nav-link" aria-label="Go to Stacks page">"Stacks"</a>
                <a href="#/settings" class="nav-link" aria-label="Go to Settings page">"Settings"</a>
                <Show when=move || !is_online.get()>
                    <span class="offline-indicator" aria-live="polite">"Offline"</span>
                </Show>
            </nav>
            <AlertBanner
                alert=alert_message
                on_dismiss=Some(Callback::new(move |_| {
                    let alerts = get_alerts(&engine::models::AlertFilter {
                        user_id: Some("local-device".to_string()),
                        acknowledged: Some(false),
                    }).unwrap_or_default();
                    if let Some(alert) = alerts.first() {
                        let _ = acknowledge_alert(&alert.id);
                    }
                    version.update(|v| *v += 1);
                }))
            />
            <main>
                {children()}
            </main>
        </div>
    }
}
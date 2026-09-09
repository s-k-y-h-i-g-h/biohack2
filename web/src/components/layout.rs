use crate::components::AlertBanner;
use crate::state::db::{acknowledge_alert, get_alerts};
use crate::state::store::AppContext;
use leptos::prelude::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_online = RwSignal::new(true);

    // Global data version — bumped by pages after writes (see AppContext)
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;
    let current_path = ctx.current_path;

    // Which nav link (href fragment) is active, for aria-current="page".
    let is_active = move |href: &str| {
        let p = current_path.get();
        if href == "#/" {
            p == "/" || p.is_empty()
        } else {
            p == href[1..]
        }
    };

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
            <nav class="nav" aria-label="Main navigation">
                <a
                    href="#/"
                    class="nav-link"
                    aria-label="Go to Log page"
                    aria-current=move || if is_active("#/") { "page" } else { "" }
                >"Log"</a>
                <a
                    href="#/history"
                    class="nav-link"
                    aria-label="Go to History page"
                    aria-current=move || if is_active("#/history") { "page" } else { "" }
                >"History"</a>
                <a
                    href="#/vitals"
                    class="nav-link"
                    aria-label="Go to Vitals page"
                    aria-current=move || if is_active("#/vitals") { "page" } else { "" }
                >"Vitals"</a>
                <a
                    href="#/notes"
                    class="nav-link"
                    aria-label="Go to Notes page"
                    aria-current=move || if is_active("#/notes") { "page" } else { "" }
                >"Notes"</a>
                <a
                    href="#/stacks"
                    class="nav-link"
                    aria-label="Go to Stacks page"
                    aria-current=move || if is_active("#/stacks") { "page" } else { "" }
                >"Stacks"</a>
                <a
                    href="#/settings"
                    class="nav-link"
                    aria-label="Go to Settings page"
                    aria-current=move || if is_active("#/settings") { "page" } else { "" }
                >"Settings"</a>
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

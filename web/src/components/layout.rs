use leptos::*;
use leptos::prelude::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_online = RwSignal::new(true);

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
            <main>
                {children()}
            </main>
        </div>
    }
}

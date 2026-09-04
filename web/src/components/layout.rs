use leptos::*;
use leptos_router::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let app_ctx = expect_context::<crate::state::store::AppContext>();
    
    view! {
        <div class="app-container">
            <nav class="nav">
                <div class="nav-brand">Biohack Tracker</div>
                <div class="nav-links">
                    <A href="/">Log</A>
                    <A href="/history">History</A>
                    <A href="/vitals">Vitals</A>
                    <A href="/stacks">Stacks</A>
                    <A href="/insights">Insights</A>
                    <A href="/settings">Settings</A>
                </div>
                <div class="offline-indicator" class:offline="!app_ctx.is_online.get()">
                    {if !app_ctx.is_online.get() { "Offline" } else { "" }}
                </div>
            </nav>
            <main class="main">
                {children()}
            </main>
        </div>
    }
}

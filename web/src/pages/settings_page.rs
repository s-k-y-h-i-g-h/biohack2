use leptos::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Settings"</h1>
            <div class="card">
                <h2>"Data Export"</h2>
                <button class="btn btn-secondary" on:click=move |_| {
                    // Export functionality
                }>
                    "Export JSON"
                </button>
            </div>
        </div>
    }
}

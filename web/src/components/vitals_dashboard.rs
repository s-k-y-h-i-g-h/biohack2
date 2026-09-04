use leptos::*;

#[component]
pub fn VitalsDashboard() -> impl IntoView {
    view! {
        <div class="vitals-dashboard">
            <h2>"Recent Vitals"</h2>
            <div class="vitals-grid">
                <div class="vital-card">
                    <span class="vital-label">"Blood Pressure"</span>
                    <span class="vital-value">"--/--"</span>
                    <span class="vital-unit">"mmHg"</span>
                </div>
                <div class="vital-card">
                    <span class="vital-label">"Heart Rate"</span>
                    <span class="vital-value">"--"</span>
                    <span class="vital-unit">"bpm"</span>
                </div>
                <div class="vital-card">
                    <span class="vital-label">"Weight"</span>
                    <span class="vital-value">"--"</span>
                    <span class="vital-unit">"kg"</span>
                </div>
            </div>
            <p class="empty-state">"No vitals logged yet."</p>
        </div>
    }
}

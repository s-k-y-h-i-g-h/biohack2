use leptos::*;
use leptos::prelude::*;
use engine::models::VitalsEntry;

#[component]
pub fn VitalsDashboard(recent_vitals: Vec<VitalsEntry>) -> impl IntoView {
    let has_vitals = !recent_vitals.is_empty();

    view! {
        <div class="vitals-dashboard">
            <Show when=move || has_vitals>
                <h3>"Recent Vitals"</h3>
                <div class="vitals-grid">
                    {recent_vitals.first().map(|v| {
                        let bp = format!("{}/{}", v.bp_systolic.unwrap_or(0), v.bp_diastolic.unwrap_or(0));
                        let hr = v.heart_rate.unwrap_or(0);
                        let spo2 = format!("{}%", v.spo2.unwrap_or(0));
                        let temp = format!("{}°C", v.temperature.unwrap_or(0.0));

                        view! {
                            <>
                                <div class="vital-item"><span class="vital-label">"BP"</span><span>{bp}</span></div>
                                <div class="vital-item"><span class="vital-label">"HR"</span><span>{hr}</span></div>
                                <div class="vital-item"><span class="vital-label">"SpO2"</span><span>{spo2}</span></div>
                                <div class="vital-item"><span class="vital-label">"Temp"</span><span>{temp}</span></div>
                            </>
                        }
                    })}
                </div>
            </Show>
            <Show when=move || !has_vitals>
                <p>"No vitals logged yet."</p>
            </Show>
        </div>
    }
}

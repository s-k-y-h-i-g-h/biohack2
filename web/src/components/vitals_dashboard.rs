use engine::models::VitalsEntry;
use leptos::prelude::*;

/// Displays the latest vitals plus recent readings. Takes a reactive signal so
/// the dashboard live-updates when a new entry is saved.
#[component]
pub fn VitalsDashboard(recent_vitals: Signal<Vec<VitalsEntry>>) -> impl IntoView {
    view! {
        <div class="vitals-dashboard">
            <h3>"Recent Vitals"</h3>
            {move || {
                let vitals = recent_vitals.get();
                if vitals.is_empty() {
                    view! {
                        <p class="empty-state">"No vitals logged yet."</p>
                    }.into_any()
                } else {
                    let latest = vitals[0].clone();
                    let recent: Vec<VitalsEntry> = vitals.iter().take(3).cloned().collect();

                    let bp = format!(
                        "{}/{}",
                        latest.bp_systolic.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string()),
                        latest.bp_diastolic.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string()),
                    );
                    let hr = latest.heart_rate.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string());
                    let sp = latest.spo2.map(|v| format!("{}%", v)).unwrap_or_else(|| "—".to_string());
                    let tp = latest.temperature.map(|v| format!("{}°C", v)).unwrap_or_else(|| "—".to_string());

                    view! {
                        <div class="vitals-grid">
                            <div class="vital-item"><span class="vital-label">"BP"</span><span>{bp}</span></div>
                            <div class="vital-item"><span class="vital-label">"HR"</span><span>{hr}</span></div>
                            <div class="vital-item"><span class="vital-label">"SpO2"</span><span>{sp}</span></div>
                            <div class="vital-item"><span class="vital-label">"Temp"</span><span>{tp}</span></div>
                        </div>
                        <div class="vitals-recent-list">
                            {recent.into_iter().map(|v| {
                                let t = v.timestamp.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string();
                                let parts: Vec<String> = [
                                    v.bp_systolic.map(|x| format!("BP {}", x)),
                                    v.heart_rate.map(|x| format!("HR {}", x)),
                                    v.weight.map(|x| format!("{}kg", x)),
                                    v.spo2.map(|x| format!("{}%", x)),
                                    v.temperature.map(|x| format!("{}°C", x)),
                                ].into_iter().flatten().collect();
                                let summary = parts.join(" · ");
                                view! {
                                    <div class="vitals-recent-item">
                                        <span class="note-timestamp">{t}</span>
                                        <span>{summary}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

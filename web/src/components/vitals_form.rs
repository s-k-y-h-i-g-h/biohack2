use leptos::*;

#[component]
pub fn VitalsForm() -> impl IntoView {
    let (bp_systolic, set_bp_systolic) = create_signal(String::new());
    let (bp_diastolic, set_bp_diastolic) = create_signal(String::new());
    let (heart_rate, set_heart_rate) = create_signal(String::new());
    let (weight, set_weight) = create_signal(String::new());
    let (temperature, set_temperature) = create_signal(String::new());
    let (spo2, set_spo2) = create_signal(String::new());
    let (sleep_quality, set_sleep_quality) = create_signal(String::new());
    let (notes, set_notes) = create_signal(String::new());
    
    view! {
        <div class="vitals-form">
            <h2>"Log Vitals"</h2>
            
            <div class="form-grid">
                <div class="form-group">
                    <label>"Blood Pressure Systolic"</label>
                    <input type="number" value=bp_systolic on:input=move |e| set_bp_systolic.set(event_target_value(&e)) placeholder="120" />
                </div>
                <div class="form-group">
                    <label>"Blood Pressure Diastolic"</label>
                    <input type="number" value=bp_diastolic on:input=move |e| set_bp_diastolic.set(event_target_value(&e)) placeholder="80" />
                </div>
                <div class="form-group">
                    <label>"Heart Rate (bpm)"</label>
                    <input type="number" value=heart_rate on:input=move |e| set_heart_rate.set(event_target_value(&e)) placeholder="72" />
                </div>
                <div class="form-group">
                    <label>"Weight (kg)"</label>
                    <input type="number" step="0.1" value=weight on:input=move |e| set_weight.set(event_target_value(&e)) placeholder="75.5" />
                </div>
                <div class="form-group">
                    <label>"Temperature (°C)"</label>
                    <input type="number" step="0.1" value=temperature on:input=move |e| set_temperature.set(event_target_value(&e)) placeholder="37.0" />
                </div>
                <div class="form-group">
                    <label>"SpO2 (%)"</label>
                    <input type="number" value=spo2 on:input=move |e| set_spo2.set(event_target_value(&e)) placeholder="98" />
                </div>
                <div class="form-group">
                    <label>"Sleep Quality"</label>
                    <select value=sleep_quality.on_change(move |v| set_sleep_quality.set(v))>
                        <option value="">Select...</option>
                        <option value="poor">Poor</option>
                        <option value="fair">Fair</option>
                        <option value="good">Good</option>
                        <option value="excellent">Excellent</option>
                    </select>
                </div>
            </div>
            
            <div class="form-group">
                <label>"Notes"</label>
                <textarea value=notes.on_change(move |v| set_notes.set(v)) placeholder="How are you feeling?" rows="3"></textarea>
            </div>
            
            <button class="btn btn-primary" on:click=move |_| {
                leptos::logging::log!("Saving vitals: {}/{} HR={}", bp_systolic.get(), bp_diastolic.get(), heart_rate.get());
            }>
                "Save Vitals"
            </button>
        </div>
    }
}

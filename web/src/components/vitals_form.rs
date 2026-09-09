use engine::models::VitalsEntry;
use leptos::prelude::*;

#[component]
pub fn VitalsForm(on_save: Callback<VitalsEntry>) -> impl IntoView {
    let bp_systolic = RwSignal::new(String::new());
    let bp_diastolic = RwSignal::new(String::new());
    let heart_rate = RwSignal::new(String::new());
    let weight = RwSignal::new(String::new());
    let spo2 = RwSignal::new(String::new());
    let temperature = RwSignal::new(String::new());
    let success = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let handle_save = move |_| {
        error.set(None);

        // At least one measurement required
        if bp_systolic.get().is_empty()
            && bp_diastolic.get().is_empty()
            && heart_rate.get().is_empty()
            && weight.get().is_empty()
            && spo2.get().is_empty()
            && temperature.get().is_empty()
        {
            error.set(Some("Enter at least one measurement".to_string()));
            return;
        }

        // Parse + validate ranges (per data model: BP 60-250, HR 20-300, SpO2 50-100, Temp 30-45°C)
        let sbp = if bp_systolic.get().is_empty() {
            None
        } else {
            match bp_systolic.get().parse::<i32>() {
                Ok(v) if (60..=250).contains(&v) => Some(v),
                _ => {
                    error.set(Some("Systolic must be between 60-250 mmHg".to_string()));
                    return;
                }
            }
        };
        let dbp = if bp_diastolic.get().is_empty() {
            None
        } else {
            match bp_diastolic.get().parse::<i32>() {
                Ok(v) if (40..=150).contains(&v) => Some(v),
                _ => {
                    error.set(Some("Diastolic must be between 40-150 mmHg".to_string()));
                    return;
                }
            }
        };
        let hr = if heart_rate.get().is_empty() {
            None
        } else {
            match heart_rate.get().parse::<i32>() {
                Ok(v) if (20..=300).contains(&v) => Some(v),
                _ => {
                    error.set(Some("Heart rate must be between 20-300 bpm".to_string()));
                    return;
                }
            }
        };
        let w = if weight.get().is_empty() {
            None
        } else {
            match weight.get().parse::<f64>() {
                Ok(v) if v > 0.0 && v < 1000.0 => Some(v),
                _ => {
                    error.set(Some("Weight must be a positive number (kg)".to_string()));
                    return;
                }
            }
        };
        let sp = if spo2.get().is_empty() {
            None
        } else {
            match spo2.get().parse::<i32>() {
                Ok(v) if (50..=100).contains(&v) => Some(v),
                _ => {
                    error.set(Some("SpO2 must be between 50-100%".to_string()));
                    return;
                }
            }
        };
        let temp = if temperature.get().is_empty() {
            None
        } else {
            match temperature.get().parse::<f64>() {
                Ok(v) if (30.0..=45.0).contains(&v) => Some(v),
                _ => {
                    error.set(Some("Temperature must be between 30-45°C".to_string()));
                    return;
                }
            }
        };

        let entry = VitalsEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "local-device".to_string(),
            timestamp: chrono::Utc::now(),
            bp_systolic: sbp,
            bp_diastolic: dbp,
            heart_rate: hr,
            weight: w,
            blood_glucose: None,
            temperature: temp,
            spo2: sp,
            hrv: None,
            sleep_quality: None,
            custom_metrics: None,
            notes: None,
        };
        on_save.run(entry);

        // Clear the form and show confirmation
        bp_systolic.set(String::new());
        bp_diastolic.set(String::new());
        heart_rate.set(String::new());
        weight.set(String::new());
        spo2.set(String::new());
        temperature.set(String::new());
        success.set(true);
        set_timeout(
            move || success.set(false),
            std::time::Duration::from_millis(2000),
        );
    };

    view! {
        <div class="vitals-form">
            <div class="form-row">
                <label for="systolic">"Systolic:"</label>
                <input
                    id="systolic"
                    type="number"
                    placeholder="120"
                    prop:value=move || bp_systolic.get()
                    on:input=move |e| { bp_systolic.set(event_target_value(&e)); }
                    aria-label="Systolic blood pressure"
                />
            </div>
            <div class="form-row">
                <label for="diastolic">"Diastolic:"</label>
                <input
                    id="diastolic"
                    type="number"
                    placeholder="80"
                    prop:value=move || bp_diastolic.get()
                    on:input=move |e| { bp_diastolic.set(event_target_value(&e)); }
                    aria-label="Diastolic blood pressure"
                />
            </div>
            <div class="form-row">
                <label for="heart_rate">"Heart Rate:"</label>
                <input
                    id="heart_rate"
                    type="number"
                    placeholder="72"
                    prop:value=move || heart_rate.get()
                    on:input=move |e| { heart_rate.set(event_target_value(&e)); }
                    aria-label="Heart rate"
                />
            </div>
            <div class="form-row">
                <label for="weight">"Weight:"</label>
                <input
                    id="weight"
                    type="number"
                    placeholder="70"
                    step="0.1"
                    prop:value=move || weight.get()
                    on:input=move |e| { weight.set(event_target_value(&e)); }
                    aria-label="Weight in kg"
                />
            </div>
            <div class="form-row">
                <label for="spo2">"SpO2:"</label>
                <input
                    id="spo2"
                    type="number"
                    placeholder="98"
                    prop:value=move || spo2.get()
                    on:input=move |e| { spo2.set(event_target_value(&e)); }
                    aria-label="Blood oxygen saturation"
                />
            </div>
            <div class="form-row">
                <label for="temperature">"Temperature:"</label>
                <input
                    id="temperature"
                    type="number"
                    placeholder="37.0"
                    step="0.1"
                    prop:value=move || temperature.get()
                    on:input=move |e| { temperature.set(event_target_value(&e)); }
                    aria-label="Temperature in Celsius"
                />
            </div>
            <button type="button" on:click=handle_save aria-label="Save vitals">"Save"</button>

            <Show when=move || success.get()>
                <div class="toast success" role="status">
                    "Vitals logged!"
                </div>
            </Show>
            <Show when=move || error.get().is_some()>
                <div class="toast error" role="alert">
                    {move || error.get()}
                </div>
            </Show>
        </div>
    }
}

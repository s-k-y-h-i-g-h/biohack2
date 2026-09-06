use leptos::*;
use leptos::prelude::*;
use engine::models::VitalsEntry;

#[component]
pub fn VitalsForm(
    on_save: Callback<VitalsEntry>,
) -> impl IntoView {
    let bp_systolic = RwSignal::new(String::new());
    let bp_diastolic = RwSignal::new(String::new());
    let heart_rate = RwSignal::new(String::new());
    let weight = RwSignal::new(String::new());
    let spo2 = RwSignal::new(String::new());
    let temperature = RwSignal::new(String::new());

    let handle_save = move |_| {
        let entry = VitalsEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "local-device".to_string(),
            timestamp: chrono::Utc::now(),
            bp_systolic: bp_systolic.get().parse().ok(),
            bp_diastolic: bp_diastolic.get().parse().ok(),
            heart_rate: heart_rate.get().parse().ok(),
            weight: weight.get().parse().ok(),
            blood_glucose: None,
            temperature: temperature.get().parse().ok(),
            spo2: spo2.get().parse().ok(),
            hrv: None,
            sleep_quality: None,
            custom_metrics: None,
            notes: None,
        };
        on_save.run(entry);
    };

    view! {
        <div class="vitals-form">
            <div class="form-row">
                <label for="systolic">"Systolic:"</label>
                <input
                    id="systolic"
                    type="number"
                    placeholder="120"
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
                    on:input=move |e| { weight.set(event_target_value(&e)); }
                    aria-label="Weight"
                />
            </div>
            <div class="form-row">
                <label for="spo2">"SpO2:"</label>
                <input
                    id="spo2"
                    type="number"
                    placeholder="98"
                    on:input=move |e| { spo2.set(event_target_value(&e)); }
                    aria-label="SpO2"
                />
            </div>
            <div class="form-row">
                <label for="temperature">"Temperature:"</label>
                <input
                    id="temperature"
                    type="number"
                    placeholder="37.0"
                    step="0.1"
                    on:input=move |e| { temperature.set(event_target_value(&e)); }
                    aria-label="Temperature"
                />
            </div>
            <button type="button" on:click=handle_save aria-label="Save vitals">"Save"</button>
        </div>
    }
}

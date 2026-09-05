use leptos::*;
use leptos::prelude::*;

#[component]
pub fn VitalsForm(
    on_save: Callback<String>,
) -> impl IntoView {
    let bp_systolic = RwSignal::new(String::new());
    let bp_diastolic = RwSignal::new(String::new());
    let heart_rate = RwSignal::new(String::new());

    let handle_save = move |_| {
        let data = format!(
            "{{\"systolic\":\"{}\",\"diastolic\":\"{}\",\"heart_rate\":\"{}\"}}",
            bp_systolic.get(),
            bp_diastolic.get(),
            heart_rate.get()
        );
        on_save.run(data);
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
            <button type="button" on:click=handle_save aria-label="Save vitals">"Save"</button>
        </div>
    }
}

use leptos::*;

#[component]
pub fn InteractionWarning() -> impl IntoView {
    let (show, set_show) = create_signal(false);
    let (message, set_message) = create_signal(String::new());
    
    view! {
        {show.get().then(|| {
            view! {
                <div class="interaction-warning">
                    <div class="warning-content">
                        <h3>"⚠️ Interaction Warning"</h3>
                        <p>{message.get()}</p>
                        <div class="warning-actions">
                            <button class="btn btn-danger" on:click=move |_| set_show.set(false)>"Acknowledge & Continue"</button>
                            <button class="btn btn-secondary" on:click=move |_| set_show.set(false)>"Cancel"</button>
                        </div>
                    </div>
                </div>
            }.into_view()
        })}
    }
}

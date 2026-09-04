use leptos::*;

#[component]
pub fn AlertBanner() -> impl IntoView {
    let (alerts, set_alerts) = create_signal(Vec::<String>::new());
    
    effect(move |_| {
        // Check for alerts based on vitals
    });
    
    view! {
        <div class="alert-banner" class:has-alerts=move || !alerts.is_empty()>
            {alerts.get().iter().map(|alert| {
                view! {
                    <div class="alert-item">
                        <span class="alert-message">{alert}</span>
                        <button class="alert-dismiss" on:click=move |_| {
                            // Dismiss alert
                        }>"×"</button>
                    </div>
                }.into_view()
            }).collect_view()}
        </div>
    }
}

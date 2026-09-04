use leptos::*;

#[component]
pub fn ErrorBoundary(children: Children) -> impl IntoView {
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // In production, this would catch and display errors
    view! {
        <div class="error-boundary">
            {error.get().map(|e| {
                view! {
                    <div class="error-message">
                        <p>"Something went wrong"</p>
                        <p class="error-detail">{e}</p>
                        <button on:click=move |_| set_error.set(None)>"Retry"</button>
                    </div>
                }.into_view()
            })}
            {children()}
        </div>
    }
}

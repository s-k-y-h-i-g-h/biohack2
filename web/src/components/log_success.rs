use leptos::*;

#[component]
pub fn LogSuccess() -> impl IntoView {
    let (show, set_show) = create_signal(false);
    
    on_event::< leptos::ev::KeyDown >(move |event| {
        if event.key() == "Escape" {
            set_show.set(false);
        }
    });
    
    view! {
        {show.get().then(|| {
            view! {
                <div class="toast success">
                    <p>"Entry logged successfully!"</p>
                    <button on:click=move |_| set_show.set(false)>"Close"</button>
                </div>
            }.into_view()
        })}
    }
}

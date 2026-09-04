use leptos::*;

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (is_dark, set_is_dark) = create_signal(true);
    
    effect(move |_| {
        if is_dark.get() {
            document::element().set_attribute("data-theme", "dark").ok();
        } else {
            document::element().set_attribute("data-theme", "light").ok();
        }
    });
    
    view! {
        <button
            class="theme-toggle"
            on:click=move |_| set_is_dark.update(|v| *v = !*v)
            aria-label="Toggle theme"
        >
            {if is_dark.get() { "☀️" } else { "🌙" }}
        </button>
    }
}

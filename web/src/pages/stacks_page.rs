use leptos::*;
use leptos::prelude::*;

#[component]
pub fn HistoryPage() -> impl IntoView {
    view! {
        <div>
            <h2>"History"</h2>
            <p>"No entries yet. Start logging!"</p>
        </div>
    }
}

#[component]
pub fn VitalsPage() -> impl IntoView {
    view! {
        <div>
            <h2>"Vitals"</h2>
            <p>"Track your blood pressure, heart rate, and more."</p>
        </div>
    }
}

#[component]
pub fn StacksPage() -> impl IntoView {
    view! {
        <div>
            <h2>"Stacks"</h2>
            <p>"Create and manage your supplement stacks."</p>
        </div>
    }
}

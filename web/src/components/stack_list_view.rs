use leptos::*;

#[component]
pub fn StackListView() -> impl IntoView {
    view! {
        <div class="stack-list">
            <h2>"Your Stacks"</h2>
            <div class="empty-state">
                <p>"No stacks created yet. Create your first stack!"</p>
            </div>
        </div>
    }
}

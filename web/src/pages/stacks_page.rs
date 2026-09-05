use leptos::*;
use leptos::prelude::*;

#[component]
pub fn StacksPage() -> impl IntoView {
    let stacks = RwSignal::new(Vec::<(String, String)>::new());
    
    view! {
        <div class="page">
            <h2>"Stacks"</h2>
            <p>"Create and manage your supplement stacks."</p>
            
            <div class="stack-builder">
                <h3>"Create New Stack"</h3>
                <input
                    type="text"
                    placeholder="Stack name"
                    aria-label="Stack name"
                />
                <button type="button">"Create Stack"</button>
            </div>
            
            <div class="stack-list">
                <h3>"Your Stacks"</h3>
                <Show when=move || stacks.get().is_empty()>
                    <p>"No stacks yet. Create one above!"</p>
                </Show>
            </div>
        </div>
    }
}

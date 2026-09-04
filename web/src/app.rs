use leptos::*;
use leptos_router::*;

use crate::components::layout::Layout;
use super::router::RouterView;

#[component]
pub fn App() -> impl IntoView {
    provide_context(crate::state::store::AppContext::new());
    
    view! {
        <Layout>
            <RouterView />
        </Layout>
    }
}

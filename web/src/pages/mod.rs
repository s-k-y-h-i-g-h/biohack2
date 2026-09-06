use leptos::*;
use leptos::prelude::*;
use engine::models::CatalogItem;
use engine::catalog::seed_catalog;
use crate::components::LogForm;

pub fn log_page() -> impl IntoView {
    let catalog = seed_catalog();

    view! {
        <div class="page">
            <h2>"Log Consumption"</h2>
            <LogForm catalog=catalog />
        </div>
    }
}

mod history_page;
pub use history_page::HistoryPage as history_page;

pub mod vitals_page;
pub use vitals_page::VitalsPage as vitals_page;

pub fn stacks_page() -> impl IntoView {
    view! {
        <div class="page">
            <h2>"Stacks"</h2>
            <p>"Create and manage your supplement stacks."</p>
            <div>"Stack management coming soon..."</div>
        </div>
    }
}

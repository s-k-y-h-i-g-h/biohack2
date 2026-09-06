use leptos::*;
use leptos::prelude::*;
use engine::models::CatalogItem;
use engine::catalog::seed_catalog;
use crate::pages::LogPage;

pub fn log_page() -> impl IntoView {
    view! {
        <LogPage />
    }
}

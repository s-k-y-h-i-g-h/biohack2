use leptos::prelude::*;

/// Global app state shared across pages via provide_context.
/// `data_version` bumps whenever any page writes to localStorage, so
/// cross-page reactive readers (e.g. Layout's alert banner) re-read storage.
#[derive(Clone)]
pub struct AppContext {
    pub user_id: String,
    pub data_version: RwSignal<u32>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            user_id: "local-device".to_string(),
            data_version: RwSignal::new(0),
        }
    }
}
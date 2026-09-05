use leptos::*;
use leptos::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub user_id: String,
    pub is_online: ReadSignal<bool>,
}

impl AppContext {
    pub fn new() -> Self {
        let is_online = RwSignal::new(true);
        
        Self {
            user_id: "local-device".to_string(),
            is_online: is_online.read_only(),
        }
    }
}

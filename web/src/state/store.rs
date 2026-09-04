use leptos::*;

#[derive(Clone)]
pub struct AppContext {
    pub user_id: String,
    pub is_online: Signal<bool>,
}

impl AppContext {
    pub fn new() -> Self {
        let (is_online, set_online) = create_signal(true);
        
        // Monitor online status
        effect(move |_| {
            let online = window().navigator().online();
            set_online.set(online);
        });
        
        Self {
            user_id: "local-device".to_string(),
            is_online,
        }
    }
}

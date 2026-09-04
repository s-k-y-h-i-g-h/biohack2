use leptos::*;

#[derive(Clone)]
pub struct AppContext {
    pub user_id: String,
    pub is_online: ReadSignal<bool>,
}

impl AppContext {
    pub fn new() -> Self {
        let (is_online, _set_online) = create_signal(true);
        
        // Set up online/offline detection
        if let Some(window) = web_sys::window() {
            if let Ok(navigator) = window.navigator() {
                if let Ok(online) = navigator.on_line() {
                    let _ = online.listen(|_| {});
                }
            }
        }
        
        Self {
            user_id: "local-device".to_string(),
            is_online,
        }
    }
}

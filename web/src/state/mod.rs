pub struct AppState {
    pub user_id: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user_id: "local-device".to_string(),
        }
    }
}

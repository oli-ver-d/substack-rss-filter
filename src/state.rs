use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub api_key: Arc<String>,
}

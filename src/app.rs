use crate::config::Config;
use axum::http::StatusCode;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok\n")
}

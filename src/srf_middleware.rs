use crate::{errors::AppError, state::AppState};
use axum::{
    Router,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, header},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;

pub async fn auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let uri_api_key = Query::<UriApiKey>::try_from_uri(req.uri())
        .map_err(|_| AppError::AuthError("Missing ?API_KEY={} in uri".to_string()))?;

    if uri_api_key.api_key == *state.api_key {
        Ok(next.run(req).await)
    } else {
        Err(AppError::AuthError("Invalid API_KEY".to_string()))
    }
}

#[derive(Deserialize)]
struct UriApiKey {
    #[serde(rename = "API_KEY")]
    api_key: String,
}

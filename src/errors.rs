use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    FetchError(String),
    ParseError(String),
    BuildError(String),
    AuthError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::FetchError(msg) => (StatusCode::BAD_REQUEST, format!("Fetch Error: {}", msg)),
            AppError::ParseError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Fetch Error: {}", msg),
            ),
            AppError::BuildError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Fetch Error: {}", msg),
            ),
            AppError::AuthError(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Authorization Error: {}", msg),
            ),
        };

        (status, message).into_response()
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Invalid username or password")]
    InvalidCredentials,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Access denied")]
    Forbidden,

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    /// A multi-step operation that only partly applied. Unlike `InternalError`, this message
    /// IS returned to the caller - so it must name the *step* that failed and never embed a
    /// raw transport error (RCON host:port, socket paths, IO details).
    #[error("{0}")]
    PartialFailure(String),
}

impl AppError {
    /// Maps the error to its HTTP status and the message actually sent to the client.
    /// Split out of `into_response` so the client-facing message can be unit-tested without
    /// building an axum `Response`.
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token has expired".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Access denied".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            // Internal messages carry host detail (RCON host:port, socket paths, IO
            // errors) - log them, never ship them to the client.
            AppError::InternalError(msg) => {
                tracing::error!("internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            // 409: the request was valid and partly took effect, so the resource is now in a
            // state that conflicts with what was asked for. The operator needs to know which
            // step did not apply, so this message is passed through verbatim.
            AppError::PartialFailure(msg) => (StatusCode::CONFLICT, msg.clone()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = self.status_and_message();

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `InternalError` detail must stay server-side; `PartialFailure` detail must reach the
    /// client - the two halves of the same rule.
    #[test]
    fn internal_detail_is_hidden_but_partial_failure_detail_is_returned() {
        let secret = "Failed to connect to RCON at 127.0.0.1:25575 (/run/user/1000/podman/podman.sock)";
        let (status, message) = AppError::InternalError(secret.to_string()).status_and_message();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "Internal server error");
        assert!(!message.contains("127.0.0.1:25575"));
        assert!(!message.contains("/run/user/1000"));

        let step = "Group 'builder' was created but setting its weight failed";
        let (status, message) = AppError::PartialFailure(step.to_string()).status_and_message();
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(message, step);
    }
}

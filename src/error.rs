// use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum ValorantError {
    #[error("Valorant lockfile not found — is Valorant running?")]
    LockfileNotFound,
    #[error("Lockfile malformed")]
    LockfileMalformed,
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("Not in a match")]
    NotInMatch,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },
}

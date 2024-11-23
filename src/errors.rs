use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProveError {
    #[error("Failed to create directory: {0}")]
    CreateDirError(#[from] std::io::Error),

    #[error("Failed to download keys: {0}")]
    DownloadKeysError(#[source] anyhow::Error),

    #[error("Failed to download compiled circuit: {0}")]
    DownloadCircuitError(#[source] anyhow::Error),

    #[error("Failed to generate proof: {0}")]
    GenerateProofError(#[source] anyhow::Error),

    #[error("Failed to read proof and public data: {0}")]
    ReadProofError(#[source] anyhow::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to unzip compiled circuit: {0}")]
    UnzipCircuitError(#[source] anyhow::Error),

    #[error("Failed to unzip keys: {0}")]
    UnzipKeysError(#[source] anyhow::Error),

    #[error("Failed to clean up: {0}")]
    Cleanup(#[source] anyhow::Error),
}

impl IntoResponse for ProveError {
    fn into_response(self) -> Response {
        // Map the error to an appropriate HTTP status code
        let status_code = match self {
            ProveError::CreateDirError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::DownloadKeysError(_) => StatusCode::BAD_REQUEST,
            ProveError::DownloadCircuitError(_) => StatusCode::BAD_REQUEST,
            ProveError::GenerateProofError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::ReadProofError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::JsonError(_) => StatusCode::BAD_REQUEST,
            ProveError::UnzipCircuitError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::UnzipKeysError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::Cleanup(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Create a JSON body with the error message
        let body = Json(serde_json::json!({
            "status": "error",
            "message": self.to_string(),
        }));

        // Build the response
        (status_code, body).into_response()
    }
}

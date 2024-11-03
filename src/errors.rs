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

    #[error("Failed to download input: {0}")]
    DownloadInputError(#[source] anyhow::Error),

    #[error("Failed to download keys: {0}")]
    DownloadKeysError(#[source] anyhow::Error),

    #[error("Failed to download compiled circuit: {0}")]
    DownloadCircuitError(#[source] anyhow::Error),

    #[error("Failed to generate proof: {0}")]
    GenerateProofError(#[source] anyhow::Error),

    #[error("Failed to upload proof: {0}")]
    UploadProofError(#[source] anyhow::Error),

    #[error("Failed to upload public data: {0}")]
    UploadPublicError(#[source] anyhow::Error),

    #[error("Failed to read proof and public data: {0}")]
    ReadProofError(#[source] anyhow::Error),

    #[error("Failed to delete directory: {0}")]
    CleanUpError(#[source] anyhow::Error),
}

impl IntoResponse for ProveError {
    fn into_response(self) -> Response {
        // Map the error to an appropriate HTTP status code
        let status_code = match self {
            ProveError::CreateDirError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::DownloadInputError(_) => StatusCode::BAD_REQUEST,
            ProveError::DownloadKeysError(_) => StatusCode::BAD_REQUEST,
            ProveError::DownloadCircuitError(_) => StatusCode::BAD_REQUEST,
            ProveError::GenerateProofError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::UploadProofError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::UploadPublicError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::ReadProofError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProveError::CleanUpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

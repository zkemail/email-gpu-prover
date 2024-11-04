use axum::{response::IntoResponse, Json};
use sdk_utils::{download_from_url, upload_to_url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    errors::ProveError,
    prove::{clean_up, prove, read_proof_and_public_data, Proof, PublicOutputs},
};

#[derive(Deserialize, Debug)]
pub struct ProveAndPushRequest {
    pub blueprint_id: String,
    pub input_download_url: String,
    pub keys_download_url: String,
    pub compiled_circuit_download_url: String,
    pub proof_upload_url: String,
    pub public_upload_url: String,
}

#[derive(Deserialize, Debug)]
pub struct ProveRequest {
    pub blueprint_id: String,
    pub input: Value,
    pub keys_download_url: String,
    pub compiled_circuit_download_url: String,
}

#[derive(Serialize, Debug)]
pub struct ProveResponse {
    pub proof: Proof,
    pub public_outputs: PublicOutputs,
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Hello from ZK Email!";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn prove_and_push_handler(
    Json(payload): Json<ProveAndPushRequest>,
) -> Result<impl IntoResponse, ProveError> {
    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all(&payload.blueprint_id)?;

    download_from_url(
        &payload.input_download_url,
        &format!("{}/input.json", payload.blueprint_id),
    )
    .await
    .map_err(ProveError::DownloadInputError)?;

    download_from_url(
        &payload.keys_download_url,
        &format!("{}/keys.zip", payload.blueprint_id),
    )
    .await
    .map_err(ProveError::DownloadKeysError)?;

    download_from_url(
        &payload.compiled_circuit_download_url,
        &format!("{}/compiled_circuit.zip", payload.blueprint_id),
    )
    .await
    .map_err(ProveError::DownloadCircuitError)?;

    prove(&payload.blueprint_id)
        .await
        .map_err(ProveError::GenerateProofError)?;

    upload_to_url(&payload.proof_upload_url, "artifacts/proof.json")
        .await
        .map_err(ProveError::UploadProofError)?;

    upload_to_url(&payload.public_upload_url, "artifacts/public.json")
        .await
        .map_err(ProveError::UploadPublicError)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Proof generated and uploaded successfully"
    })))
}

pub async fn prove_handler(
    Json(payload): Json<ProveRequest>,
) -> Result<impl IntoResponse, ProveError> {
    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all(&payload.blueprint_id)?;

    // Write the input to a file
    let input_file = format!("{}/input.json", payload.blueprint_id);
    std::fs::write(&input_file, serde_json::to_string(&payload.input)?)?;

    download_from_url(
        &payload.keys_download_url,
        &format!("{}/keys.zip", payload.blueprint_id),
    )
    .await
    .map_err(ProveError::DownloadKeysError)?;

    download_from_url(
        &payload.compiled_circuit_download_url,
        &format!("{}/compiled_circuit.zip", payload.blueprint_id),
    )
    .await
    .map_err(ProveError::DownloadCircuitError)?;

    prove(&payload.blueprint_id)
        .await
        .map_err(ProveError::GenerateProofError)?;

    let (proof, public) =
        read_proof_and_public_data(&payload.blueprint_id).map_err(ProveError::ReadProofError)?;

    clean_up(&payload.blueprint_id).map_err(ProveError::CleanUpError)?;

    Ok(Json(ProveResponse {
        proof,
        public_outputs: public,
    }))
}

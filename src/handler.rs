use axum::{response::IntoResponse, Json};
use relayer_utils::LOG;
use sdk_utils::download_from_url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slog::info;

use crate::{
    errors::ProveError,
    prove::{clean_up, prove, read_proof_and_public_data, Proof, PublicOutputs},
};
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProveRequest {
    pub blueprint_id: String,
    pub input: Value,
    pub keys_download_url: String,
    pub compiled_circuit_download_url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
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

pub async fn prove_handler(
    Json(payload): Json<ProveRequest>,
) -> Result<impl IntoResponse, ProveError> {
    info!(LOG, "Received request to prove a circuit"; "blueprint_id" => &payload.blueprint_id);

    let proof_path = format!("artifacts/{}", payload.blueprint_id);
    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all(&proof_path)?;

    info!(LOG, "Created artifact folder"; "path" => &proof_path);

    // Write the input to a file
    let input_file = format!("{}/input.json", proof_path);
    std::fs::write(&input_file, serde_json::to_string(&payload.input)?)?;

    info!(LOG, "Wrote input to file"; "path" => &input_file);

    download_from_url(
        &payload.compiled_circuit_download_url,
        &format!("{}/compiled_circuit.zip", proof_path),
    )
    .await
    .map_err(ProveError::DownloadCircuitError)?;

    info!(LOG, "Downloaded compiled circuit");

    download_from_url(
        &payload.keys_download_url,
        &format!("{}/keys.zip", proof_path),
    )
    .await
    .map_err(ProveError::DownloadKeysError)?;

    info!(LOG, "Downloaded keys");

    prove(&proof_path)
        .await
        .map_err(ProveError::GenerateProofError)?;

    info!(LOG, "Generated proof");

    let (proof, public) =
        read_proof_and_public_data(&proof_path).map_err(ProveError::ReadProofError)?;

    info!(LOG, "Read proof and public data");

    clean_up(&proof_path).map_err(ProveError::CleanUpError)?;

    info!(LOG, "Cleaned up artifacts");

    Ok(Json(ProveResponse {
        proof,
        public_outputs: public,
    }))
}

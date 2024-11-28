use axum::{response::IntoResponse, Json};
use relayer_utils::LOG;
use sdk_utils::{download_from_url, run_command};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slog::info;

use crate::{
    errors::ProveError,
    prove::{cleanup, prove, read_proof_and_public_data, Proof, PublicOutputs},
};
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProveRequest {
    pub blueprint_id: String,
    pub proof_id: String,
    pub input: Value,
    pub zkey_download_url: String,
    pub circuit_cpp_download_url: String,
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
    let blueprint_path = format!("artifacts/{}", payload.blueprint_id);

    // Check if the artifact folder already exists
    if !std::path::Path::new(&blueprint_path).exists() {
        std::fs::create_dir_all(&blueprint_path)?;

        download_from_url(
            &payload.circuit_cpp_download_url,
            &format!("{}/circuit_cpp.zip", blueprint_path),
        )
        .await
        .map_err(ProveError::DownloadCircuitError)?;

        info!(LOG, "Downloaded compiled circuit");

        download_from_url(
            &payload.zkey_download_url,
            &format!("{}/zkey.zip", blueprint_path),
        )
        .await
        .map_err(ProveError::DownloadKeysError)?;

        info!(LOG, "Downloaded keys");

        // Unzip compiled circuit into the artifacts folder
        info!(LOG, "Unzipping compiled circuit");
        run_command("unzip", &["-o", "circuit_cpp.zip"], Some(&blueprint_path))
            .await
            .map_err(ProveError::UnzipCircuitError)?;

        // Unzip keys files into the artifacts folder
        info!(LOG, "Unzipping keys");
        run_command("unzip", &["-o", "zkey.zip"], Some(&blueprint_path))
            .await
            .map_err(ProveError::UnzipKeysError)?;
    }

    // Write the input to a file
    let input_file = format!("{}/input_{}.json", blueprint_path, payload.proof_id);
    std::fs::write(&input_file, serde_json::to_string(&payload.input)?)?;

    info!(LOG, "Wrote input to file"; "path" => &input_file);

    prove(&blueprint_path, &payload.proof_id)
        .await
        .map_err(ProveError::GenerateProofError)?;

    info!(LOG, "Generated proof");

    let (proof, public) = read_proof_and_public_data(&blueprint_path, &payload.proof_id)
        .map_err(ProveError::ReadProofError)?;

    info!(LOG, "Read proof and public data");

    cleanup(&blueprint_path, &payload.proof_id).map_err(ProveError::Cleanup)?;

    Ok(Json(ProveResponse {
        proof,
        public_outputs: public,
    }))
}

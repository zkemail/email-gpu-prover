use anyhow::Result;
use relayer_utils::LOG;
use sdk_utils::run_command;
use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Deserialize, Serialize, Debug)]
pub struct Proof {
    pub pi_a: Vec<String>,
    pub pi_b: Vec<Vec<String>>,
    pub pi_c: Vec<String>,
    pub protocol: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct PublicOutputs(pub Vec<String>);

pub async fn prove(artifacts_dir: &str) -> Result<()> {
    info!(LOG, "artifacts_dir: {}", artifacts_dir);

    // Unzip compiled circuit into the artifacts folder
    info!(LOG, "Unzipping compiled circuit");
    if let Err(e) = run_command(
        "unzip",
        &["-o", "compiled_circuit.zip"],
        Some(artifacts_dir),
    )
    .await
    {
        info!(LOG, "Error unzipping compiled circuit: {:?}", e);
        return Err(e);
    }

    // Unzip keys files into the artifacts folder
    info!(LOG, "Unzipping keys");
    if let Err(e) = run_command("unzip", &["-o", "keys.zip"], Some(artifacts_dir)).await {
        info!(LOG, "Error unzipping keys: {:?}", e);
        return Err(e);
    }

    // Generate witness
    info!(LOG, "Generating witness");
    if let Err(e) = run_command(
        "./circuit",
        &["input.json", "witness.wtns"],
        Some(artifacts_dir),
    )
    .await
    {
        info!(LOG, "Error generating witness: {:?}", e);
        return Err(e);
    }

    // Generate the proof
    info!(LOG, "Generating proof");
    run_command(
        "prover",
        &["circuit.zkey", "witness.wtns", "proof.json", "public.json"],
        Some(artifacts_dir),
    )
    .await?;

    Ok(())
}

pub fn read_proof_and_public_data(artifacts_dir: &str) -> Result<(Proof, PublicOutputs)> {
    let proof = std::fs::read_to_string(format!("{}/proof.json", artifacts_dir))?;
    let public = std::fs::read_to_string(format!("{}/public.json", artifacts_dir))?;

    let proof: Proof = serde_json::from_str(&proof)?;
    let public: PublicOutputs = serde_json::from_str(&public)?;

    Ok((proof, public))
}

pub fn clean_up(artifacts_dir: &str) -> Result<()> {
    info!(LOG, "Cleaning up artifacts");
    std::fs::remove_dir_all(artifacts_dir)?;

    Ok(())
}

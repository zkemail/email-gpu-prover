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

pub async fn prove(blueprint_path: &str) -> Result<()> {
    info!(LOG, "blueprint_path: {}", blueprint_path);

    // Generate witness
    info!(LOG, "Generating witness");
    run_command(
        "./circuit",
        &["input.json", "witness.wtns"],
        Some(blueprint_path),
    )
    .await?;

    // Generate the proof
    info!(LOG, "Generating proof");
    run_command(
        "prover",
        &["circuit.zkey", "witness.wtns", "proof.json", "public.json"],
        Some(blueprint_path),
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

use std::process::Command;

use utoipa::OpenApi as _;

use submerge_crystal::api::docs::APIDoc;

const SPEC_FILE_JSON_PATH: &str = "api-spec/submerge-crystal-api.json";
const CRYSTAL_CLIENT_OUTPUT_PATH: &str = "src/api/v1/client";

fn has_oas3_gen() -> bool {
    std::process::Command::new("oas3-gen")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn install_oas3_gen() -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args(["install", "oas3-gen"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to install oas3-gen");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Generates OPENAPI specs v3.1.0
    let openapi = APIDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).unwrap();
    std::fs::write(SPEC_FILE_JSON_PATH, json)?;
    println!("OpenAPI specification written to {SPEC_FILE_JSON_PATH}.");

    // Generates Rust client for the spec file
    if !has_oas3_gen() {
        println!("oas3-gen not found, installing...");
        install_oas3_gen()?;
    }

    let status = Command::new("oas3-gen")
        .args([
            "generate",
            "-i",
            SPEC_FILE_JSON_PATH,
            "-o",
            CRYSTAL_CLIENT_OUTPUT_PATH,
            "client-mod",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("oas3-gen generate failed");
    }

    Ok(())
}

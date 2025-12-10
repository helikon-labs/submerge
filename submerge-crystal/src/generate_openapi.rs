use utoipa::OpenApi as _;

use submerge_crystal::api::docs::APIDoc;

const SPEC_FILE_JSON_PATH: &str = "api-spec/submerge-crystal-api.json";

fn main() -> anyhow::Result<()> {
    let openapi = APIDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).unwrap();
    std::fs::write(SPEC_FILE_JSON_PATH, json)?;
    println!("OpenAPI specification written to {SPEC_FILE_JSON_PATH}.");
    Ok(())
}

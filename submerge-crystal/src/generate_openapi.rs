use utoipa::OpenApi as _;

use submerge_crystal::api::APIDoc;

fn main() -> anyhow::Result<()> {
    let openapi = APIDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi).unwrap();
    std::fs::write("api-spec/submerge-crystal-api.yaml", yaml)?;
    println!("OpenAPI specification written to api-spec/submerge-crystal-api.yaml.");
    Ok(())
}

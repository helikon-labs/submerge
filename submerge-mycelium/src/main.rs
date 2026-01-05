use submerge_crystal::api::v1::client::{GetBlocksRequest, SubmergeCrystalApiV1Client};

#[tokio::main]
async fn main() {
    let client = SubmergeCrystalApiV1Client::new();
    let response = client
        .get_blocks(GetBlocksRequest {
            ..Default::default()
        })
        .await
        .unwrap();

    println!("Blocks: {:#?}", response);
}

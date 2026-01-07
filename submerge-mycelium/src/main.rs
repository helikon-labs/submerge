use submerge_crystal::api::v1::client::{BlocksRequest, SubmergeCrystalApiV1Client};

#[tokio::main]
async fn main() {
    let client = SubmergeCrystalApiV1Client::new();
    let response = client
        .blocks(BlocksRequest {
            ..Default::default()
        })
        .await
        .unwrap();

    println!("Blocks: {:#?}", response);
}

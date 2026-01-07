use submerge_crystal::api::v1::client::SubmergeCrystalApiV1Client;

pub fn get_api_client() -> SubmergeCrystalApiV1Client {
    SubmergeCrystalApiV1Client::new()
}

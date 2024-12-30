use lazy_static::lazy_static;
use submerge_base::BaseService;
use submerge_crystal::Crystal;

lazy_static! {
    static ref SERVICE: Crystal = Crystal;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

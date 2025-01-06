use once_cell::sync::OnceCell;
use submerge_base::BaseService;
use submerge_crystal::Crystal;

static SERVICE: OnceCell<Crystal> = OnceCell::new();

#[tokio::main]
async fn main() {
    let _ = SERVICE.set(Crystal::new().await.unwrap());
    SERVICE.get().unwrap().start().await;
}

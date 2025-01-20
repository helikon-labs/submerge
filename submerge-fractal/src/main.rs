use lazy_static::lazy_static;
use submerge_base::BaseService;
use submerge_fractal::Fractal;

lazy_static! {
    static ref SERVICE: Fractal = Fractal;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

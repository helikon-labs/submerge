use axum::Router;
use axum_embed::ServeEmbed;

#[derive(Clone, rust_embed::Embed)]
#[folder = "./admin/dist"]
struct Admin;

pub(crate) fn admin_router() -> Router {
    let path = "/admin".to_string();
    let index = "index.html".to_string();
    let assets = ServeEmbed::<Admin>::with_parameters(
        Some(format!("{path}/{index}")),
        axum_embed::FallbackBehavior::Ok,
        Some(index),
    );
    Router::new().nest_service(&path, assets)
}

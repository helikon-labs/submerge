use crate::api::v1::{
    genesis::get_genesis_records,
    metadata::{
        get_metadata_hex, get_metadata_json, get_metadata_list, get_metadata_pallet_calls,
        get_metadata_pallet_constants, get_metadata_pallet_errors, get_metadata_pallet_events,
        get_metadata_pallet_storage_items, get_metadata_pallets,
    },
    system::{cancel_all_workers, get_worker_ids, spawn_worker},
};
use crate::metrics;
use crate::worker::WorkerManager;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use submerge_base::{args::PostgreSQLArgs, types::substrate::chainspec::ChainProperties};
use submerge_persistence::postgres::PostgreSQLStorage;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

pub mod legacy;
mod v1;
pub mod validation;

#[derive(Clone)]
pub struct ServiceState {
    pub chain_properties: ChainProperties,
    pub postgres: Arc<PostgreSQLStorage>,
    pub worker_manager: Arc<WorkerManager>,
}

pub(crate) async fn on_server_ready(host: &str, port: u16) {
    log::info!("🌐 HTTP API started on {host}:{port}.");
}

async fn metrics_middleware(request: Request, next: Next) -> Response {
    metrics::api_requests_total().inc();
    metrics::api_active_connections().inc();

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let status_code = response.status();

    metrics::api_response_time_ms().observe(elapsed);
    metrics::api_response_status_code_counter()
        .with_label_values(&[status_code.as_str()])
        .inc();
    metrics::api_active_connections().dec();

    response
}

fn build_api_routes() -> Router<ServiceState> {
    Router::new()
        // system
        .route(
            "/system/workers",
            get(get_worker_ids)
                .post(spawn_worker)
                .delete(cancel_all_workers),
        )
        // metadata
        .route("/metadata", get(get_metadata_list))
        .route("/metadata/{spec_version}/json", get(get_metadata_json))
        .route("/metadata/{spec_version}/hex", get(get_metadata_hex))
        .route(
            "/metadata/{spec_version}/pallets",
            get(get_metadata_pallets),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}/calls",
            get(get_metadata_pallet_calls),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}/constants",
            get(get_metadata_pallet_constants),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}/errors",
            get(get_metadata_pallet_errors),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}/events",
            get(get_metadata_pallet_events),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}/storage",
            get(get_metadata_pallet_storage_items),
        )
        // genesis
        .route("/genesis", get(get_genesis_records))
        .layer(middleware::from_fn(metrics_middleware))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler for the API.");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler for the API.")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    log::info!("🛑 Shutdown signal received, starting graceful shutdown.");
}

pub(crate) async fn run_api(
    chain_properties: ChainProperties,
    postgres_args: &PostgreSQLArgs,
    worker_manager: &Arc<WorkerManager>,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    let postgres = Arc::new(PostgreSQLStorage::new(postgres_args).await?);
    let service_state = ServiceState {
        chain_properties,
        postgres: postgres.clone(),
        worker_manager: worker_manager.clone(),
    };
    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:3000"),
            //HeaderValue::from_static("https://yourdomain.com"),
        ])
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .nest("/v1", build_api_routes())
        .with_state(service_state)
        .layer(cors);
    let listener = TcpListener::bind((host, port)).await?;
    let server = axum::serve(listener, app);
    let graceful_server = server.with_graceful_shutdown(shutdown_signal());
    let (server_result, _) = tokio::join!(graceful_server, on_server_ready(host, port));
    server_result?;
    Ok(())
}

use crate::api::v1::call::{
    get_call_args_by_hash, get_call_by_hash, get_calls, get_calls_by_block_reference,
    get_calls_by_block_reference_and_extrinsic_index, get_calls_by_extrinsic_hash,
    get_parent_call_by_hash, get_sub_calls_by_hash,
};
use crate::api::v1::event::{
    get_event_args_by_hash, get_event_by_hash, get_events, get_events_by_block_reference,
    get_events_by_block_reference_and_extrinsic_index, get_events_by_block_reference_and_index,
    get_events_by_extrinsic_hash,
};
use crate::api::v1::extrinsic::{
    get_extrinsic_by_hash, get_extrinsic_root_call_by_hash,
    get_extrinsics_by_block_reference_and_index,
};
use crate::api::v1::trace::{get_traces, get_traces_by_block_reference};
use crate::metrics;
use crate::worker::WorkerManager;
use crate::{
    api::v1::{
        block::{get_blocks, get_blocks_by_reference},
        extrinsic::{get_extrinsics, get_extrinsics_by_block_reference},
        genesis::get_genesis_records,
        metadata::{
            get_metadata, get_metadata_hex, get_metadata_json, get_metadata_list,
            get_metadata_pallet, get_metadata_pallet_calls, get_metadata_pallet_constants,
            get_metadata_pallet_errors, get_metadata_pallet_events,
            get_metadata_pallet_storage_items, get_metadata_pallets,
        },
        // system::{cancel_all_workers, get_worker_ids, spawn_worker},
    },
    types::api::error::APIError,
};
use axum::body::Body;
use axum::extract::Request;
use axum::http::header;
use axum::http::HeaderValue;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use submerge_base::args::PostgreSQLArgs;
use submerge_metrics::use_metric;
use submerge_persistence::postgres::PostgreSQLStorage;
use tokio::net::TcpListener;
use tokio::signal;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::cors::{Any, CorsLayer};

pub mod legacy;
mod v1;

const MAX_RESPONSE_MESSAGE_BYTES: usize = 64 * 1024;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ServiceState {
    pub chain_name: String,
    pub postgres: Arc<PostgreSQLStorage>,
    pub worker_manager: Arc<WorkerManager>,
}

pub(crate) async fn on_server_ready(host: &str, port: u16) {
    tracing::info!("🌐 HTTP API started on {host}:{port}.");
}

async fn metrics_middleware(request: Request, next: Next) -> Response {
    use_metric(metrics::api_requests_total(), |m| m.inc());
    use_metric(metrics::api_active_connections(), |m| m.inc());

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let status_code = response.status();

    use_metric(metrics::api_response_time_ms(), |m| m.observe(elapsed));
    use_metric(metrics::api_response_status_code_counter(), |m| {
        m.with_label_values(&[status_code.as_str()]).inc();
    });
    use_metric(metrics::api_active_connections(), |m| m.dec());
    response
}

async fn json_error_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let status = response.status();
    let is_error = status.is_client_error() || status.is_server_error();
    if !is_error {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    parts.headers.remove(header::CONTENT_LENGTH);
    let bytes = axum::body::to_bytes(body, MAX_RESPONSE_MESSAGE_BYTES)
        .await
        .unwrap_or_default();
    if serde_json::from_slice::<serde_json::Value>(&bytes).is_ok() {
        parts
            .headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("application/json"));
        return Response::from_parts(parts, Body::from(bytes));
    }

    let original_msg = String::from_utf8_lossy(&bytes).trim().to_string();

    let error_key = status
        .canonical_reason()
        .unwrap_or("Error")
        .to_ascii_lowercase()
        .replace(' ', "_");

    let message = if original_msg.is_empty() {
        format!("{}.", error_key.replace('_', " "))
    } else {
        original_msg
    };

    let body = serde_json::json!({
        "message": message,
    });
    parts.headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    if let Ok(body) = serde_json::to_vec(&body) {
        Response::from_parts(parts, Body::from(body))
    } else {
        APIError::SerializationError.into_response()
    }
}

fn build_api_routes() -> Router<ServiceState> {
    Router::new()
        // system
        /*.route(
            "/system/workers",
            get(get_worker_ids)
                .post(spawn_worker)
                .delete(cancel_all_workers),
        )*/
        // metadata
        .route("/metadata", get(get_metadata_list))
        .route("/metadata/{spec_version}", get(get_metadata))
        .route("/metadata/{spec_version}/json", get(get_metadata_json))
        .route("/metadata/{spec_version}/hex", get(get_metadata_hex))
        .route(
            "/metadata/{spec_version}/pallets",
            get(get_metadata_pallets),
        )
        .route(
            "/metadata/{spec_version}/pallets/{pallet_index}",
            get(get_metadata_pallet),
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
        // blocks
        .route("/blocks", get(get_blocks))
        .route("/blocks/{block_ref}", get(get_blocks_by_reference))
        // events
        .route("/events", get(get_events))
        .route(
            "/blocks/{block_ref}/events",
            get(get_events_by_block_reference),
        )
        .route(
            "/blocks/{block_ref}/events/{index}",
            get(get_events_by_block_reference_and_index),
        )
        .route(
            "/blocks/{block_ref}/extrinsics/{extrinsic_index}/events",
            get(get_events_by_block_reference_and_extrinsic_index),
        )
        .route(
            "/extrinsics/{extrinsic_hash}/events",
            get(get_events_by_extrinsic_hash),
        )
        .route("/events/{event_hash}", get(get_event_by_hash))
        .route("/events/{event_hash}/args", get(get_event_args_by_hash))
        // extrinsics
        .route("/extrinsics", get(get_extrinsics))
        .route(
            "/blocks/{block_ref}/extrinsics",
            get(get_extrinsics_by_block_reference),
        )
        .route(
            "/blocks/{block_ref}/extrinsics/{index}",
            get(get_extrinsics_by_block_reference_and_index),
        )
        .route("/extrinsics/{extrinsic_hash}", get(get_extrinsic_by_hash))
        .route(
            "/extrinsics/{extrinsic_hash}/call",
            get(get_extrinsic_root_call_by_hash),
        )
        // genesis
        .route("/genesis", get(get_genesis_records))
        // calls
        .route("/calls", get(get_calls))
        .route(
            "/blocks/{block_ref}/calls",
            get(get_calls_by_block_reference),
        )
        .route(
            "/blocks/{block_ref}/extrinsics/{extrinsic_index}/calls",
            get(get_calls_by_block_reference_and_extrinsic_index),
        )
        .route(
            "/extrinsics/{extrinsic_hash}/calls",
            get(get_calls_by_extrinsic_hash),
        )
        .route("/calls/{call_hash}", get(get_call_by_hash))
        .route("/calls/{call_hash}/args", get(get_call_args_by_hash))
        .route("/calls/{call_hash}/subs", get(get_sub_calls_by_hash))
        .route("/calls/{call_hash}/parent", get(get_parent_call_by_hash))
        // traces
        .route("/traces", get(get_traces))
        .route(
            "/blocks/{block_ref}/traces",
            get(get_traces_by_block_reference),
        )
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
    tracing::info!("🛑 Shutdown signal received, starting graceful shutdown.");
}

pub(crate) async fn run_api(
    chain_name: String,
    postgres_args: &PostgreSQLArgs,
    worker_manager: &Arc<WorkerManager>,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    let postgres = Arc::new(PostgreSQLStorage::new(postgres_args).await?);
    let service_state = ServiceState {
        chain_name,
        postgres: postgres.clone(),
        worker_manager: worker_manager.clone(),
    };
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .unwrap();
    let governor_limiter = governor_conf.limiter().clone();
    let interval = std::time::Duration::from_secs(60);
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        tracing::info!("rate limiting storage size: {}", governor_limiter.len());
        governor_limiter.retain_recent();
    });

    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("https://docs.submerge.io"),
        ])
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .nest("/api/v1", build_api_routes())
        .fallback(|| async { APIError::NotFound })
        .with_state(service_state)
        .layer(cors)
        .layer(GovernorLayer::new(governor_conf))
        .layer(middleware::from_fn(json_error_middleware))
        .layer(middleware::from_fn(metrics_middleware));
    let listener = TcpListener::bind((host, port)).await?;
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    );
    let graceful_server = server.with_graceful_shutdown(shutdown_signal());
    let (server_result, _) = tokio::join!(graceful_server, on_server_ready(host, port));
    server_result?;
    Ok(())
}

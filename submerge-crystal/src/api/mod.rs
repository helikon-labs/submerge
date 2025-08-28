use crate::api::v1::metadata::{
    get_metadata_hex, get_metadata_json, get_metadata_list, get_metadata_pallet_calls,
    get_metadata_pallet_constants, get_metadata_pallet_errors, get_metadata_pallet_events,
    get_metadata_pallet_storage_items, get_metadata_pallets,
};
use crate::metrics;
use crate::types::api::error::APIError;
use actix_web::dev::Service as _;
use actix_web::{web, HttpResponse};
use actix_web::{App, HttpServer};
use futures_util::future::FutureExt;
use std::sync::Arc;
use submerge_base::args::PostgreSQLArgs;
use submerge_persistence::postgres::PostgreSQLStorage;

pub mod legacy;
mod v1;

type APIResult = Result<HttpResponse, APIError>;

#[derive(Clone)]
pub struct ServiceState {
    pub postgres: Arc<PostgreSQLStorage>,
}

pub(crate) async fn on_server_ready(host: &str, port: u16) {
    log::info!("🌐 HTTP API started on {host}:{port}.");
}

pub(crate) async fn run_api(
    postgres_args: &PostgreSQLArgs,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    let postgres = Arc::new(PostgreSQLStorage::new(postgres_args).await?);
    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
            ])
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(ServiceState {
                postgres: postgres.clone(),
            }))
            .wrap_fn(|request, service| {
                metrics::api_requests_total().inc();
                metrics::api_active_connections().inc();
                let start = std::time::Instant::now();
                service.call(request).map(move |result| {
                    match &result {
                        Ok(response) => {
                            let status_code = response.response().status();
                            metrics::api_response_time_ms()
                                .observe(start.elapsed().as_millis() as f64);
                            metrics::api_response_status_code_counter()
                                .with_label_values(&[status_code.as_str()])
                                .inc();
                        }
                        Err(error) => {
                            let status_code = error.as_response_error().status_code();
                            metrics::api_response_time_ms()
                                .observe(start.elapsed().as_millis() as f64);
                            metrics::api_response_status_code_counter()
                                .with_label_values(&[status_code.as_str()])
                                .inc();
                        }
                    }
                    metrics::api_active_connections().dec();
                    result
                })
            })
            .service(
                web::scope("v1")
                    .service(get_metadata_list)
                    .service(get_metadata_json)
                    .service(get_metadata_hex)
                    .service(get_metadata_pallets)
                    .service(get_metadata_pallet_calls)
                    .service(get_metadata_pallet_constants)
                    .service(get_metadata_pallet_errors)
                    .service(get_metadata_pallet_events)
                    .service(get_metadata_pallet_storage_items),
            )
    })
    .workers(10)
    .disable_signals()
    .bind(format!("{host}:{port}"))?
    .run();
    let (server_result, _) = tokio::join!(server, on_server_ready(host, port));
    Ok(server_result?)
}

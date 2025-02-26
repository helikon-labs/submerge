use crate::persistence::PostgreSQLStorage;
use crate::{get_postgres, metrics};
use actix_web::dev::Service as _;
use actix_web::{get, http::header::ContentType, web, HttpResponse};
use actix_web::{App, HttpServer};
use futures_util::future::FutureExt;
use serde::Deserialize;
use std::sync::Arc;
use submerge_base::args::PostgreSQLArgs;
use submerge_base::err::{InternalServerError, ServiceError};
use submerge_types::submerge::BlockTraces;
use submerge_types::substrate::BLOCK_HASH_HEX_LENGTH;

pub(crate) type ResultResponse = Result<HttpResponse, InternalServerError>;

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
    let postgres = Arc::new(get_postgres(postgres_args).await?);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ServiceState {
                postgres: postgres.clone(),
            }))
            .wrap_fn(|request, service| {
                metrics::request_counter().inc();
                metrics::open_connection_count().inc();
                let start = std::time::Instant::now();
                service.call(request).map(move |result| {
                    match &result {
                        Ok(response) => {
                            let status_code = response.response().status();
                            metrics::response_time_ms().observe(start.elapsed().as_millis() as f64);
                            metrics::response_status_code_counter(status_code.as_str()).inc();
                        }
                        Err(error) => {
                            let status_code = error.as_response_error().status_code();
                            metrics::response_time_ms().observe(start.elapsed().as_millis() as f64);
                            metrics::response_status_code_counter(status_code.as_str()).inc();
                        }
                    }
                    metrics::open_connection_count().dec();
                    result
                })
            })
            .service(hello_world)
            .service(get_block_traces)
    })
    .workers(10)
    .disable_signals()
    .bind(format!("{}:{}", host, port,))?
    .run();
    let (server_result, _) = tokio::join!(server, on_server_ready(host, port));
    Ok(server_result?)
}

#[get("/hello_world")]
pub(crate) async fn hello_world() -> ResultResponse {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("Hello, world!"))
}

#[derive(Deserialize)]
pub(crate) struct BlockHashOrNumberParameter {
    block_hash_or_number: String,
}

#[get("/block/{block_hash_or_number}/trace")]
pub(crate) async fn get_block_traces(
    path: web::Path<BlockHashOrNumberParameter>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    match path.block_hash_or_number.parse::<u64>() {
        Ok(block_number) => Ok(HttpResponse::Ok().json(
            data.postgres
                .get_block_traces_by_number(block_number)
                .await?,
        )),
        Err(_) => {
            let input = path.block_hash_or_number.trim_start_matches("0x");
            if input.len() < BLOCK_HASH_HEX_LENGTH {
                Ok(HttpResponse::BadRequest()
                    .json(ServiceError::from("Invalid block hash or number.")))
            } else {
                match hex::decode(input) {
                    Ok(block_hash) => {
                        match data.postgres.get_block_traces_by_hash(&block_hash).await? {
                            Some(block_traces) => Ok(HttpResponse::Ok().json(vec![block_traces])),
                            None => Ok(HttpResponse::Ok().json(Vec::<BlockTraces>::new())),
                        }
                    }
                    Err(_) => Ok(HttpResponse::BadRequest()
                        .json(ServiceError::from("Invalid block hash or number."))),
                }
            }
        }
    }
}

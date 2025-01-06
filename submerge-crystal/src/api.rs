use actix_web::dev::Service as _;
use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web::{App, HttpServer};
use futures_util::future::FutureExt;
use submerge_base::err::InternalServerError;

use crate::metrics;

pub(crate) type ResultResponse = Result<HttpResponse, InternalServerError>;

pub(crate) async fn on_server_ready(host: &str, port: u16) {
    log::info!("🌐 HTTP API started on {host}:{port}.");
}

pub(crate) async fn run_api(host: &str, port: u16) -> anyhow::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
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

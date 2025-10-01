use crate::service::Vote;
use crate::types::ReferendumDTO;
use actix_cors::Cors;
use actix_web::{dev::Service as _, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use dv_report_config::Config;
use dv_report_persistence::postgres::PostgreSQLStorage;
use dv_report_service::err::InternalServerError;
use dv_report_service::Service;
use dv_report_types::dv::delegate::Delegate;
use dv_report_types::substrate::account_id::AccountId;
use futures_util::future::FutureExt;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Instant;

mod metrics;
mod service;
mod types;

pub(crate) type ResultResponse = Result<HttpResponse, InternalServerError>;

const CACHE_LIFETIME_MS: u64 = 5 * 60 * 1000;
const CACHE_MAX_CAPACITY: u64 = 1000;

async fn on_server_ready() {
    log::info!("HTTP service started.");
}

#[derive(Clone)]
pub(crate) struct ServiceState {
    network_referendum_cache: Arc<Cache<u32, Vec<ReferendumDTO>>>,
    network_cohort_referendum_cache: Arc<Cache<(u32, u32), Vec<ReferendumDTO>>>,
    network_voter_vote_cache: Arc<Cache<(u32, AccountId), Vec<Vote>>>,
    delegate_cache: Arc<Cache<u32, Vec<Delegate>>>,
    postgres: Arc<PostgreSQLStorage>,
}

#[derive(Debug, Default)]
pub struct APIService {
    config: Config,
}

fn build_cache<A, B>() -> Cache<A, B>
where
    A: std::hash::Hash + Send + Eq + Sync + 'static,
    B: Clone + Send + Sync + 'static,
{
    Cache::builder()
        .time_to_live(std::time::Duration::from_millis(CACHE_LIFETIME_MS))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build()
}

#[async_trait(?Send)]
impl Service for APIService {
    fn name(&self) -> String {
        "API Service".to_string()
    }

    fn get_metrics_server_addr(&self) -> (String, u16) {
        (
            self.config.metrics.host.clone(),
            self.config.metrics.api_service_port,
        )
    }

    async fn run(&self) -> anyhow::Result<()> {
        let postgres = Arc::new(PostgreSQLStorage::new(&self.config).await?);
        let network_referendum_cache = Arc::new(build_cache());
        let network_cohort_referendum_cache = Arc::new(build_cache());
        let network_voter_vote_cache = Arc::new(build_cache());
        let delegate_cache = Arc::new(build_cache());
        log::info!(
            "Starting HTTP service @ {}:{}.",
            self.config.api.service_host,
            self.config.api.api_service_port
        );
        let server = HttpServer::new(move || {
            let _cors = Cors::default()
                .allowed_origin("http://localhost:8080")
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .supports_credentials();

            App::new()
                .app_data(web::Data::new(ServiceState {
                    postgres: postgres.clone(),
                    network_referendum_cache: network_referendum_cache.clone(),
                    network_cohort_referendum_cache: network_cohort_referendum_cache.clone(),
                    network_voter_vote_cache: network_voter_vote_cache.clone(),
                    delegate_cache: delegate_cache.clone(),
                }))
                //.wrap(cors)
                .wrap_fn(|request, service| {
                    metrics::request_counter().inc();
                    metrics::connection_count().inc();
                    let start = Instant::now();
                    service.call(request).map(move |result| {
                        match &result {
                            Ok(response) => {
                                let status_code = response.response().status();
                                metrics::response_time_ms()
                                    .observe(start.elapsed().as_millis() as f64);
                                metrics::response_status_code_counter(status_code.as_str()).inc();
                            }
                            Err(error) => {
                                let status_code = error.as_response_error().status_code();
                                metrics::response_time_ms()
                                    .observe(start.elapsed().as_millis() as f64);
                                metrics::response_status_code_counter(status_code.as_str()).inc();
                            }
                        }
                        metrics::connection_count().dec();
                        result
                    })
                })
                .service(service::get_all_networks)
                .service(service::get_all_referendum_statuses)
                .service(service::get_all_referendum_tracks)
                .service(service::get_all_cohorts)
                .service(service::get_all_network_cohort_tracks)
                .service(service::get_all_delegate_types)
                .service(service::get_all_delegates)
                .service(service::get_network_referenda)
                .service(service::get_network_cohort_referenda)
                .service(service::get_network_voter_votes)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            self.config.api.service_host, self.config.api.api_service_port,
        ))?
        .run();
        let (server_result, _) = tokio::join!(server, on_server_ready());
        Ok(server_result?)
    }
}

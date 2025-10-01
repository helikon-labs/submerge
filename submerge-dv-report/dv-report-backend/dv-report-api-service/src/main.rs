use dv_report_api_service::APIService;
use dv_report_config::Config;
use dv_report_service::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = APIService::default();
    let config = Config::default();
    dv_report_logging::init(&config);
    Supervisor::new(service, config.common.recovery_retry_seconds) // 10 second retry delay
        .start()
        .await
}

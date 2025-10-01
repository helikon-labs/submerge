use dv_report_config::Config;
use dv_report_indexer::Indexer;
use dv_report_service::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Indexer::default();
    let config = Config::default();
    dv_report_logging::init(&config);
    Supervisor::new(service, config.common.recovery_retry_seconds) // 10 second retry delay
        .start()
        .await
}

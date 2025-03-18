#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use log::LevelFilter;

pub mod args;
pub mod err;
pub mod types;

#[async_trait]
pub trait BaseService {
    fn get_metrics_server_addr(&self) -> Option<(String, u16)>;

    fn get_sleep_secs(&self) -> u64;

    fn get_name(&self) -> String;

    async fn run(&self) -> anyhow::Result<()>;

    async fn start(&self) {
        submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
        println!(
            r#"
███████╗██╗   ██╗██████╗ ███╗   ███╗███████╗██████╗  ██████╗ ███████╗
██╔════╝██║   ██║██╔══██╗████╗ ████║██╔════╝██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██████╔╝██╔████╔██║█████╗  ██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██╔══██╗██║╚██╔╝██║██╔══╝  ██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝██████╔╝██║ ╚═╝ ██║███████╗██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
{} v{} • © Helikon 2025"#,
            self.get_name(),
            env!("CARGO_PKG_VERSION"),
        );
        log::info!("⚙️ Starting service.");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Some(metrics_server_addr) = self.get_metrics_server_addr() {
            tokio::spawn(submerge_metrics::server::start(metrics_server_addr));
        } else {
            log::info!("⛔ Metrics disabled.");
        }
        let sleep_seconds = self.get_sleep_secs();
        loop {
            let result = self.run().await;
            if let Err(error) = result {
                log::error!("{:?}", error);
                log::warn!("Process exited. Will restart in {} seconds.", sleep_seconds,);
                tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds)).await;
            } else {
                log::info!("Process completed.");
                break;
            }
        }
    }
}

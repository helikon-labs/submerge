#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use log::LevelFilter;

pub mod err;

#[async_trait(?Send)]
pub trait BaseService {
    fn get_metrics_server_addr() -> Option<(&'static str, u16)>;

    fn get_sleep_secs() -> u64;

    async fn run(&'static self) -> anyhow::Result<()>;

    async fn start(&'static self) {
        submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
        println!(
            r#"
███████╗██╗   ██╗██████╗ ███╗   ███╗███████╗██████╗  ██████╗ ███████╗
██╔════╝██║   ██║██╔══██╗████╗ ████║██╔════╝██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██████╔╝██╔████╔██║█████╗  ██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██╔══██╗██║╚██╔╝██║██╔══╝  ██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝██████╔╝██║ ╚═╝ ██║███████╗██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
Submerge v{} © Helikon 2025"#,
            env!("CARGO_PKG_VERSION"),
        );
        log::info!("⚙️ Starting service.");
        if let Some(metrics_server_addr) = Self::get_metrics_server_addr() {
            tokio::spawn(submerge_metrics::server::start(metrics_server_addr));
        } else {
            log::info!("⛔ Metrics disabled.");
        }
        let sleep_seconds = Self::get_sleep_secs();
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

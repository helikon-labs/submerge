#![warn(clippy::disallowed_types)]

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio::{select, signal};

pub mod args;
pub mod err;
pub mod types;

#[async_trait(?Send)]
pub trait BaseService {
    fn get_name(&self) -> String;
    fn get_metrics_server_addr(&self) -> (String, u16);
    async fn run(&self) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct Supervisor<S: BaseService> {
    service: Arc<S>,
    retry_delay: Duration,
    enable_metrics: bool,
    shutdown_notify: Option<Arc<Notify>>,
}

impl<S: BaseService> Supervisor<S> {
    pub fn new(service: S, retry_delay_secs: u64) -> Self {
        Self {
            service: Arc::new(service),
            retry_delay: Duration::from_secs(retry_delay_secs),
            enable_metrics: true,
            shutdown_notify: None,
        }
    }

    pub fn with_shutdown_notify(mut self, notify: Arc<Notify>) -> Self {
        self.shutdown_notify = Some(notify);
        self
    }

    pub fn without_metrics(mut self) -> Self {
        self.enable_metrics = false;
        self
    }

    pub async fn start(self) -> anyhow::Result<()> {
        if self.enable_metrics {
            let (host, port) = self.service.get_metrics_server_addr();
            tokio::spawn(async move {
                submerge_metrics::server::start((host, port)).await;
            });
        } else {
            log::info!("⛔ Metrics disabled.");
        }
        println!(
            r#"
███████╗██╗   ██╗██████╗ ███╗   ███╗███████╗██████╗  ██████╗ ███████╗
██╔════╝██║   ██║██╔══██╗████╗ ████║██╔════╝██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██████╔╝██╔████╔██║█████╗  ██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██╔══██╗██║╚██╔╝██║██╔══╝  ██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝██████╔╝██║ ╚═╝ ██║███████╗██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
Supervisor started for {} v{} • © Helikon 2025"#,
            self.service.get_name(),
            env!("CARGO_PKG_VERSION"),
        );
        let shutdown_notify = self.shutdown_notify.clone();
        let service = self.service.clone();
        let retry_delay = self.retry_delay;
        let shutdown_signal = async {
            select! {
                _ = signal::ctrl_c() => {
                    log::warn!("Received Ctrl+C.");
                },
                _ = async {
                    if let Some(n) = &shutdown_notify {
                        n.notified().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    log::warn!("Received internal shutdown notification.");
                }
            }
        };
        let run_loop = async {
            loop {
                match service.run().await {
                    Ok(_) => {
                        log::info!("`{}` exited successfully.", service.get_name());
                        break Ok(());
                    }
                    Err(e) => {
                        log::error!("`{}` failed: {e:?}", service.get_name());
                        log::warn!("Retrying `{}` after {:?}", service.get_name(), retry_delay);
                        sleep(retry_delay).await;
                    }
                }
            }
        };

        select! {
            result = run_loop => {
                service.shutdown().await?;
                result
            }
            _ = shutdown_signal => {
                log::info!("Shutting down service `{}`...", service.get_name());
                service.shutdown().await?;
                Ok(())
            }
        }
    }
}

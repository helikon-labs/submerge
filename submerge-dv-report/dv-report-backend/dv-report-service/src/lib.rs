#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{select, signal, sync::Notify, time::sleep};

pub mod err;

#[async_trait(?Send)]
pub trait Service {
    fn name(&self) -> String;
    fn get_metrics_server_addr(&self) -> (String, u16);
    async fn run(&self) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct Supervisor<S: Service> {
    service: Arc<S>,
    retry_delay: Duration,
    enable_metrics: bool,
    shutdown_notify: Option<Arc<Notify>>,
}

impl<S: Service> Supervisor<S> {
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
                dv_report_metrics::server::start((host, port)).await;
            });
        }
        log::info!("Supervisor started for: {}", self.service.name());
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
                        log::info!("`{}` exited successfully.", service.name());
                        break Ok(());
                    }
                    Err(e) => {
                        log::error!("`{}` failed: {e:?}", service.name());
                        log::warn!("Retrying `{}` after {:?}", service.name(), retry_delay);
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
                log::info!("Shutting down service `{}`...", service.name());
                service.shutdown().await?;
                Ok(())
            }
        }
    }
}

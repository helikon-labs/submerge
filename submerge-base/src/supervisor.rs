use anyhow::Context as _;
use std::sync::Arc;
use std::time::Duration;
use tokio::{select, signal, sync::Notify, time::sleep};
use tracing::{span, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

use crate::service::BaseService;

pub struct Supervisor<S: BaseService> {
    service: Arc<S>,
    retry_delay: Duration,
    shutdown_notify: Option<Arc<Notify>>,
}

impl<S: BaseService> Supervisor<S> {
    pub fn new(service: S, retry_delay_secs: u64) -> Self {
        Self {
            service: Arc::new(service),
            retry_delay: Duration::from_secs(retry_delay_secs),
            shutdown_notify: None,
        }
    }

    pub fn with_shutdown_notify(mut self, notify: Arc<Notify>) -> Self {
        self.shutdown_notify = Some(notify);
        self
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let log_filter = self.service.get_log_env_filter()?;
        let tracing_subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_span_events(FmtSpan::ACTIVE)
            .with_target(true)
            .with_env_filter(log_filter)
            .finish();
        tracing::subscriber::set_global_default(tracing_subscriber)
            .context("Setting global default tracing subscriber failed.")?;
        let _tracing_span = span!(Level::TRACE, "Submerge Supervisor");
        let (host, port) = self.service.get_metrics_server_addr();
        tokio::spawn(async move {
            submerge_metrics::server::start((host, port)).await;
        });
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
                    tracing::warn!("⛔ Received Ctrl+C.");
                },
                _ = async {
                    if let Some(n) = &shutdown_notify {
                        n.notified().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    tracing::warn!("⛔ Received internal shutdown notification.");
                }
            }
        };
        let run_loop = async {
            loop {
                match service.run().await {
                    Ok(_) => {
                        tracing::info!("`{}` exited successfully.", service.get_name());
                        break Ok(());
                    }
                    Err(e) => {
                        tracing::error!("`{}` failed: {e:?}", service.get_name());
                        tracing::warn!("Retrying `{}` after {:?}", service.get_name(), retry_delay);
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
                tracing::info!("⛔ Shutting down service `{}`...", service.get_name());
                service.shutdown().await?;
                Ok(())
            }
        }
    }
}

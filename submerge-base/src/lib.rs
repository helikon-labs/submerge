#![warn(clippy::disallowed_types)]

use anyhow::Context as _;
use async_trait::async_trait;
use convert_case::{Case, Casing};
use std::sync::Arc;
use std::time::Duration;
use tokio::{select, signal, sync::Notify, time::sleep};
use tracing::{span, Level};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter, FmtSubscriber};

pub mod args;
pub mod types;

#[async_trait(?Send)]
pub trait BaseService {
    fn get_workspace_packages() -> Vec<String> {
        vec![
            "submerge-api".to_string(),
            "submerge-auth3".to_string(),
            "submerge-base".to_string(),
            "submerge-bloom".to_string(),
            "submerge-cli".to_string(),
            "submerge-cortex".to_string(),
            "submerge-crystal".to_string(),
            "submerge-fractal".to_string(),
            "submerge-metrics".to_string(),
            "submerge-mycelium".to_string(),
            "submerge-persistence".to_string(),
            "submerge-reflex".to_string(),
            "submerge-sentinel".to_string(),
            "submerge-substrate_client".to_string(),
            "submerge-util".to_string(),
            "submerge-web".to_string(),
        ]
    }

    fn get_name(&self) -> String;
    fn get_metrics_server_addr(&self) -> (String, u16);
    async fn run(&self) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_native_log_level(&self) -> &str;
    fn get_external_log_level(&self) -> &str;
    fn get_log_env_filter(&self) -> anyhow::Result<EnvFilter> {
        let mut filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(self.get_external_log_level()));
        let native_log_level = self.get_native_log_level();
        for package in Self::get_workspace_packages() {
            let directive = format!("{}={}", package.to_case(Case::Snake), native_log_level);
            filter = filter.add_directive(directive.parse()?);
        }
        // additional configuration
        /*
        filter = filter
            .add_directive("sqlx=debug".parse()?);
        */
        Ok(filter)
    }
}

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

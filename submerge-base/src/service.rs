use async_trait::async_trait;
use convert_case::{Case, Casing};
use tracing_subscriber::EnvFilter;

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

use axum::{extract::State, Json};
use submerge_substrate_client::RPCConfig;

use crate::{
    api::ServiceState,
    worker::{WorkerConfig, WorkerType},
};

pub(crate) async fn get_worker_ids(State(state): State<ServiceState>) -> Json<Vec<String>> {
    Json(
        state
            .worker_manager
            .get_ids()
            .await
            .iter()
            .map(|uuid| uuid.to_string())
            .collect(),
    )
}

pub(crate) async fn spawn_worker(State(state): State<ServiceState>) {
    state
        .worker_manager
        .spawn(
            WorkerType::SubscribeNewBlocks,
            WorkerConfig::new(
                state.chain_name.clone(),
                state.postgres.clone(),
                RPCConfig {
                    rpc_url: "wss://public-rpc.mainnet.aventus.io".to_string(),
                    //rpc_url: "wss://rpc.helikon.io/polkadot".to_string(),
                    rpc_connection_timeout_secs: 30,
                    rpc_request_timeout_secs: 30,
                    rpc_subscription_timeout_secs: 60,
                },
                None,
                std::time::Duration::from_secs(5),
                true,
                true,
            ),
        )
        .await;
}

pub(crate) async fn cancel_all_workers(State(state): State<ServiceState>) {
    state.worker_manager.cancel_all().await;
}

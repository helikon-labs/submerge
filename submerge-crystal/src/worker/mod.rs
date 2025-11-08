use std::{sync::Arc, time::Duration};
use submerge_substrate_client::{RPCConfig, SubstrateClient};
use tokio::sync::RwLock;

use rustc_hash::FxHashMap as HashMap;
use submerge_persistence::postgres::PostgreSQLStorage;
use tokio_util::sync::CancellationToken;
use uuid::Uuid as UUID;

use crate::{types::BlockStatus, worker::processor::BlockProcessor};

mod processor;
mod subscription;

pub enum WorkerType {
    ProcessFinalizedRange {
        maybe_start_block_number: Option<u64>,
        maybe_end_block_number: Option<u64>,
        scan: bool,
        reindex: bool,
    },
    SubscribeFinalizedBlocks,
    SubscribeNewBlocks,
}

pub struct WorkerConfig {
    chain_name: String,
    postgres: Arc<PostgreSQLStorage>,
    rpc_config: RPCConfig,
    legacy_decode_api_url: Option<String>,
    retry_delay: Duration,
    skip_traces: bool,
    stop_on_error: bool,
}

impl WorkerConfig {
    pub fn new(
        chain_name: String,
        postgres: Arc<PostgreSQLStorage>,
        rpc_config: RPCConfig,
        legacy_decode_api_url: Option<String>,
        retry_delay: Duration,
        skip_traces: bool,
        stop_on_error: bool,
    ) -> Self {
        Self {
            chain_name,
            postgres,
            rpc_config,
            legacy_decode_api_url,
            retry_delay,
            skip_traces,
            stop_on_error,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub enum WorkerStatus {
    #[default]
    Idle,
    Running {
        last_processed_block_number: Option<u64>,
        processing_block_number: Option<u64>,
        target_block_number: Option<u64>,
    },
    Error {
        last_processed_block_number: Option<u64>,
        error: Arc<anyhow::Error>,
    },
    Cancelled {
        last_processed_block_number: Option<u64>,
    },
    Completed {
        last_processed_block_number: Option<u64>,
    },
}

impl WorkerStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, WorkerStatus::Running { .. })
    }
}

pub struct Worker {
    chain_name: String,
    id: UUID,
    ty: WorkerType,
    config: WorkerConfig,
    status: RwLock<WorkerStatus>,
    cancellation_token: CancellationToken,
}

impl Worker {
    pub fn new(chain_name: String, id: UUID, ty: WorkerType, config: WorkerConfig) -> Self {
        Self {
            chain_name,
            id,
            ty,
            config,
            status: RwLock::new(Default::default()),
            cancellation_token: CancellationToken::new(),
        }
    }

    async fn set_status(&self, state: WorkerStatus) {
        *self.status.write().await = state;
    }

    pub async fn get_status(&self) -> WorkerStatus {
        self.status.read().await.clone()
    }

    async fn process_subscription(&self, block_status: BlockStatus) {
        loop {
            match self
                .subscribe_to_blocks(block_status, self.config.skip_traces)
                .await
            {
                Ok(()) => {
                    tracing::info!("{block_status} block subscription cancelled.");
                    self.set_status(WorkerStatus::Cancelled {
                        last_processed_block_number: None,
                    })
                    .await;
                    break;
                }
                Err(error) => {
                    tracing::error!(
                        "🔴 {block_status} block subscription exited with error: {error}"
                    );
                    self.set_status(WorkerStatus::Error {
                        last_processed_block_number: None,
                        error: Arc::new(error),
                    })
                    .await;
                    if self.config.stop_on_error {
                        break;
                    }
                    tracing::error!(
                        "🔄 {block_status} block subscription will retry after {} seconds.",
                        self.config.retry_delay.as_secs()
                    );
                    tokio::time::sleep(self.config.retry_delay).await;
                }
            }
        }
    }

    async fn check_rpc_compatibility(&self) -> anyhow::Result<()> {
        tracing::info!("🤖🔍 Check RPC endpoint compatibility.");
        let substrate_client = SubstrateClient::new(&self.config.rpc_config).await?;
        let health = substrate_client.get_system_health().await?;
        if health.is_syncing {
            anyhow::bail!(
                "🔴 RPC endpoint at {} is still syncing.",
                &self.config.rpc_config.rpc_url
            );
        }
        let chain_name = substrate_client.get_chain_name().await?;
        if chain_name != self.config.chain_name {
            anyhow::bail!(
                "🔴 RPC endpoint at {} is not compatible with the chain spec. Chain name mismatch: expected {}, got {}.",
                &self.config.rpc_config.rpc_url,
                &self.config.chain_name,
                chain_name,
            );
        }
        if !self.config.skip_traces {
            let Some(block_1_hash) = substrate_client.get_block_hash(1).await? else {
                anyhow::bail!("Block 1 not found on the RPC node.");
            };
            if substrate_client
                .get_block_trace(&block_1_hash)
                .await
                .is_err()
            {
                anyhow::bail!(
                    "🔴 RPC endpoint at {} does not support traces.",
                    &self.config.rpc_config.rpc_url
                );
            }
        }
        tracing::info!("🤖✅ RPC endpoint is compatible with worker configuration.");
        Ok(())
    }

    pub async fn start(&self) {
        tracing::info!("Start worker {}.", self.id);
        if let Err(error) = self.check_rpc_compatibility().await {
            tracing::error!("{error}");
            tracing::error!("🔴 RPC endpoint is incompatible. Worker is exiting.");
            self.set_status(WorkerStatus::Error {
                last_processed_block_number: None,
                error: error.into(),
            })
            .await;
            return;
        }
        match self.ty {
            WorkerType::SubscribeNewBlocks => {
                self.process_subscription(BlockStatus::Proposed).await
            }
            WorkerType::SubscribeFinalizedBlocks => {
                self.process_subscription(BlockStatus::Finalized).await
            }
            WorkerType::ProcessFinalizedRange {
                maybe_start_block_number,
                maybe_end_block_number,
                scan,
                reindex,
            } => loop {
                let block_processor = match BlockProcessor::new(
                    &self.chain_name,
                    self.id,
                    self.config.postgres.clone(),
                    &self.config.rpc_config,
                    &self.config.legacy_decode_api_url,
                )
                .await
                {
                    Ok(block_processor) => Arc::new(block_processor),
                    Err(error) => {
                        tracing::error!("🔴 Error while constructing the block processor for new block subscription: {error:?}");
                        if self.config.stop_on_error {
                            break;
                        }
                        tracing::error!(
                            "🔄 Will retry after {} seconds.",
                            self.config.retry_delay.as_secs()
                        );
                        tokio::time::sleep(self.config.retry_delay).await;
                        continue;
                    }
                };
                match block_processor
                    .process_finalized_blocks_in_range(
                        self.config.stop_on_error,
                        self.config.skip_traces,
                        scan,
                        reindex,
                        maybe_start_block_number,
                        maybe_end_block_number,
                    )
                    .await
                {
                    Ok(()) => break,
                    Err(error) => {
                        tracing::error!("🔴 Error while processing finalized blocks {maybe_start_block_number:?}-{maybe_end_block_number:?}: {error:?}");
                        if self.config.stop_on_error {
                            break;
                        }
                        tracing::error!(
                            "🔄 Will retry after {} seconds.",
                            self.config.retry_delay.as_secs()
                        );
                        tokio::time::sleep(self.config.retry_delay).await;
                        continue;
                    }
                };
            },
        }
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }
}

#[derive(Default)]
pub struct WorkerManager {
    workers: RwLock<HashMap<UUID, Arc<Worker>>>,
}

#[allow(dead_code)]
impl WorkerManager {
    pub async fn get_ids(&self) -> Vec<UUID> {
        let map = self.workers.read().await;
        map.keys().copied().collect()
    }

    pub async fn spawn(&self, ty: WorkerType, config: WorkerConfig) {
        let worker_id = UUID::new_v4();
        let worker = Arc::new(Worker::new(
            config.chain_name.clone(),
            worker_id,
            ty,
            config,
        ));
        let worker_clone = worker.clone();
        tokio::spawn(async move {
            worker_clone.start().await;
        });
        self.workers.write().await.insert(worker_id, worker);
    }

    pub async fn cancel(&self, id: UUID) -> anyhow::Result<()> {
        let workers = self.workers.read().await;
        let worker = workers
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;
        if !worker.get_status().await.is_running() {
            anyhow::bail!("Worker is not running");
        }
        worker.cancel();
        Ok(())
    }

    pub async fn remove_terminated(&self) -> Vec<UUID> {
        let ids = self.get_ids().await;
        let mut terminated_ids = Vec::new();
        for id in ids {
            if let Some(status) = self.get_status(id).await {
                if !status.is_running() {
                    terminated_ids.push(id);
                }
            }
        }
        let mut removed_ids = Vec::new();
        let mut workers = self.workers.write().await;
        for terminated_id in terminated_ids {
            if workers.remove(&terminated_id).is_some() {
                removed_ids.push(terminated_id);
            }
        }
        removed_ids
    }

    pub async fn cancel_all(&self) {
        let workers = self.workers.read().await;
        for worker in workers.values() {
            if worker.get_status().await.is_running() {
                tracing::info!("Stop worker {}.", worker.id);
                worker.cancel();
            }
        }
    }

    pub async fn get_status(&self, id: UUID) -> Option<WorkerStatus> {
        let workers = self.workers.read().await;
        if let Some(worker) = workers.get(&id) {
            Some(worker.get_status().await)
        } else {
            None
        }
    }
}

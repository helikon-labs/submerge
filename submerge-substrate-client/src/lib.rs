use anyhow::Context;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use jsonrpsee::tokio::time::timeout;
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee_core::client::{Client, ClientT, Subscription, SubscriptionClientT};
use jsonrpsee_core::rpc_params;
use parity_scale_codec::Decode;
use std::future::Future;
use std::time::Duration;
use submerge_base::types::substrate::account_id::AccountId;
use submerge_base::types::substrate::block::{Block, BlockHeader, BlockWrapper};
use submerge_base::types::substrate::block_trace::{BlockTrace, BlockTraceWrapper, StorageMethod};
use submerge_base::types::substrate::chainspec::ChainProperties;
use submerge_base::types::substrate::runtime::LastRuntimeUpgradeInfo;
use submerge_base::types::substrate::SystemHealth;
use submerge_util::substrate::storage::{decode_hex_string, get_rpc_storage_plain_params};
use tokio_util::sync::CancellationToken;

pub struct RPCConfig {
    pub rpc_url: String,
    pub rpc_connection_timeout_secs: u64,
    pub rpc_request_timeout_secs: u64,
    pub rpc_subscription_timeout_secs: u64,
}

pub struct SubstrateClient {
    ws_client: Client,
}

impl SubstrateClient {
    pub async fn new(config: &RPCConfig) -> anyhow::Result<Self> {
        Self::new_inner(
            &config.rpc_url,
            config.rpc_connection_timeout_secs,
            config.rpc_request_timeout_secs,
        )
        .await
    }

    async fn new_inner(
        rpc_url: &str,
        connection_timeout_secs: u64,
        request_timeout_secs: u64,
    ) -> anyhow::Result<Self> {
        log::info!("⚙️ Constructing Substrate client.");
        let ws_client = WsClientBuilder::default()
            .max_response_size(1024 * 1024 * 1024)
            .connection_timeout(std::time::Duration::from_secs(connection_timeout_secs))
            .request_timeout(std::time::Duration::from_secs(request_timeout_secs))
            .build(rpc_url)
            .await?;
        log::info!("✅ Substrate client constructed.");
        Ok(SubstrateClient { ws_client })
    }
}

impl SubstrateClient {
    pub async fn get_current_block_hash(&self) -> anyhow::Result<String> {
        let hash = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!())
            .await?;
        Ok(hash)
    }

    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!(block_number))
            .await?;
        Ok(hash.trim_start_matches("0x").to_string())
    }

    pub async fn get_finalized_block_hash(&self) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getFinalizedHead", rpc_params!())
            .await?;
        Ok(hash.trim_start_matches("0x").to_string())
    }

    pub async fn get_block_timestamp(&self, block_hash: &str) -> anyhow::Result<u64> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Timestamp", "Now", Some(block_hash))?,
            )
            .await?;
        decode_hex_string(hex_string.as_str())
    }

    pub async fn get_system_health(&self) -> anyhow::Result<SystemHealth> {
        let system_health: SystemHealth = self
            .ws_client
            .request("system_health", rpc_params!())
            .await?;
        Ok(system_health)
    }

    pub async fn get_chain_properties(&self) -> anyhow::Result<ChainProperties> {
        let system_health: ChainProperties = self
            .ws_client
            .request("system_properties", rpc_params!())
            .await?;
        Ok(system_health)
    }

    pub async fn get_block_header(&self, block_hash: &str) -> anyhow::Result<BlockHeader> {
        let mut header: BlockHeader = self
            .ws_client
            .request("chain_getHeader", rpc_params!(&block_hash))
            .await?;
        header.parent_hash = header.parent_hash.trim_start_matches("0x").to_string();
        header.extrinsics_root = header.extrinsics_root.trim_start_matches("0x").to_string();
        header.state_root = header.state_root.trim_start_matches("0x").to_string();
        Ok(header)
    }

    pub async fn get_block_trace(&self, block_hash: &str) -> anyhow::Result<BlockTrace> {
        let storage_method_names = StorageMethod::names().join(",");
        let trace_wrapper: BlockTraceWrapper = self
            .ws_client
            .request(
                "state_traceBlock",
                rpc_params!(&block_hash, "state", "", storage_method_names),
            )
            .await?;
        Ok(trace_wrapper.block_trace)
    }

    pub async fn get_block(&self, block_hash: &str) -> anyhow::Result<Block> {
        let block_wrapper: BlockWrapper = self
            .ws_client
            .request("chain_getBlock", rpc_params!(block_hash))
            .await?;
        Ok(block_wrapper.block)
    }

    pub async fn get_block_weight_bytes(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let maybe_hex_string: Option<String> = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("System", "BlockWeight", Some(block_hash))?,
            )
            .await?;
        if let Some(hex_string) = maybe_hex_string {
            Ok(Some(hex::decode(hex_string.trim_start_matches("0x"))?))
        } else {
            Ok(None)
        }
    }

    async fn subscribe_to_blocks<F, C>(
        &self,
        subscribe_method_name: &str,
        unsubscribe_method_name: &str,
        subscription_timeout: Duration,
        cancellation_token: CancellationToken,
        mut callback: C,
    ) -> anyhow::Result<()>
    where
        C: FnMut(BlockHeader) -> F + Send,
        F: Future<Output = anyhow::Result<()>> + Send,
    {
        let mut subscription: Subscription<BlockHeader> = self
            .ws_client
            .subscribe(
                subscribe_method_name,
                rpc_params!(),
                unsubscribe_method_name,
            )
            .await
            .map_err(|error| {
                let message = format!("⚠️ Error while subscribing to blocks: {error:?}");
                log::error!("{message}");
                anyhow::anyhow!(message)
            })?;

        loop {
            let next_item = timeout(subscription_timeout, subscription.next());
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    log::info!("🚫 Block subscription cancelled.");
                    return Ok(());
                }
                result = next_item => {
                    match result {
                        Ok(Some(Ok(block_header))) => {
                            if let Err(error) = callback(block_header).await {
                                let message = format!("⚠️ Error in callback: {error:?}");
                                log::error!("{message}");
                                return Err(anyhow::anyhow!(error)).context(message);
                            }
                        }
                        Ok(Some(Err(error))) => {
                            let message = format!("⚠️ Error while receiving block header: {error:?}");
                            log::error!("{message}");
                            return Err(anyhow::anyhow!(error)).context(message);
                        }
                        Ok(None) => {
                            let message = "⚠️ Empty block header.";
                            return Err(anyhow::anyhow!(message));
                        }
                        Err(_) => {
                            return Err(anyhow::anyhow!("⚠️ Block subscription timed out after {} sec.", subscription_timeout.as_secs()));
                        }
                    }
                }
            }
        }
    }

    pub async fn subscribe_to_new_blocks<F, C>(
        &self,
        subscription_timeout: Duration,
        cancellation_token: CancellationToken,
        callback: C,
    ) -> anyhow::Result<()>
    where
        C: FnMut(BlockHeader) -> F + Send,
        F: Future<Output = anyhow::Result<()>> + Send,
    {
        self.subscribe_to_blocks(
            "chain_subscribeNewHeads",
            "chain_unsubscribeNewHeads",
            subscription_timeout,
            cancellation_token,
            callback,
        )
        .await
    }

    /// Subscribes to finalized blocks.
    pub async fn subscribe_to_finalized_blocks<F, C>(
        &self,
        subscription_timeout: Duration,
        cancellation_token: CancellationToken,
        callback: C,
    ) -> anyhow::Result<()>
    where
        C: FnMut(BlockHeader) -> F + Send,
        F: Future<Output = anyhow::Result<()>> + Send,
    {
        self.subscribe_to_blocks(
            "chain_subscribeFinalizedHeads",
            "chain_unsubscribeFinalizedHeads",
            subscription_timeout,
            cancellation_token,
            callback,
        )
        .await
    }

    pub async fn get_last_runtime_upgrade_info(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<LastRuntimeUpgradeInfo> {
        let upgrade_info: LastRuntimeUpgradeInfo = self
            .ws_client
            .request("state_getRuntimeVersion", rpc_params!(block_hash))
            .await?;
        Ok(upgrade_info)
    }

    pub async fn get_metadata_hex_string_at_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<String> {
        let metadata_hex_string: String = self
            .ws_client
            .request("state_getMetadata", rpc_params!(block_hash))
            .await?;
        Ok(metadata_hex_string.trim_start_matches("0x").to_string())
    }

    pub async fn get_metadata_at_block(&self, block_hash: &str) -> anyhow::Result<RuntimeMetadata> {
        let metadata_hex_string = self.get_metadata_hex_string_at_block(block_hash).await?;
        let mut metadata_hex_decoded: &[u8] = &hex::decode(metadata_hex_string)?;
        let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_hex_decoded)?;
        Ok(metadata.1)
    }

    pub async fn get_active_validator_account_ids(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<AccountId>> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "Validators", Some(block_hash))?,
            )
            .await?;
        let account_ids: Vec<AccountId> = decode_hex_string(hex_string.as_str())?;
        Ok(account_ids)
    }

    pub async fn get_current_session_index(&self, block_hash: &str) -> anyhow::Result<u32> {
        let maybe_hex_string: Option<String> = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "CurrentIndex", Some(block_hash))?,
            )
            .await?;
        if let Some(hex_string) = maybe_hex_string {
            decode_hex_string(hex_string.as_str())
        } else {
            Ok(0)
        }
    }

    pub async fn get_block_event_bytes(&self, block_hash: &str) -> anyhow::Result<Vec<u8>> {
        let events_hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("System", "Events", Some(block_hash))?,
            )
            .await?;
        Ok(hex::decode(events_hex_string.trim_start_matches("0x"))?)
    }
}

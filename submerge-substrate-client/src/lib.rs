#![deny(missing_docs)]

//! ## Submerge Substrate RPC Client

use anyhow::Context;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use jsonrpsee::tokio::time::timeout;
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee_core::client::{Client, ClientT, Subscription, SubscriptionClientT};
use jsonrpsee_core::rpc_params;
use parity_scale_codec::Decode;
use std::future::Future;
use std::str::FromStr;
use std::time::Duration;
use submerge_base::types::substrate::block::{Block, BlockHeader, BlockWrapper};
use submerge_base::types::substrate::block_trace::{BlockTrace, BlockTraceWrapper, StorageMethod};
use submerge_base::types::substrate::chainspec::ChainProperties;
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_base::types::substrate::runtime::LastRuntimeUpgradeInfo;
use submerge_base::types::substrate::system::SystemHealth;
use submerge_util::substrate::storage::{decode_hex_string, get_rpc_storage_plain_params};
use tokio_util::sync::CancellationToken;

/// RPC configuration for the Substrate RPC client
#[derive(Clone)]
pub struct RPCConfig {
    /// RPC server URL
    pub rpc_url: String,
    /// Connection timeout duration is seconds
    pub rpc_connection_timeout_secs: u64,
    /// Request timeout duration is seconds
    pub rpc_request_timeout_secs: u64,
    /// Subscription timeout duration is seconds
    pub rpc_subscription_timeout_secs: u64,
}

/// Submerge Substrate Client struct
pub struct SubstrateClient {
    ws_client: Client,
}

impl SubstrateClient {
    /// Constructs a new client instance.
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
        tracing::info!("⚙️ Constructing Substrate client.");
        let ws_client = WsClientBuilder::default()
            .max_response_size(1024 * 1024 * 1024)
            .connection_timeout(std::time::Duration::from_secs(connection_timeout_secs))
            .request_timeout(std::time::Duration::from_secs(request_timeout_secs))
            .build(rpc_url)
            .await?;
        tracing::info!("✅ Substrate client constructed.");
        Ok(SubstrateClient { ws_client })
    }
}

impl SubstrateClient {
    /// Returns the hash best block hash in hexadecimal format.
    pub async fn get_current_block_hash(&self) -> anyhow::Result<String> {
        let hash = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!())
            .await?;
        Ok(hash)
    }

    /// Returns the hash of a block by its number.
    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<Option<String>> {
        let maybe_hash: Option<String> = self
            .ws_client
            .request("chain_getBlockHash", rpc_params!(block_number))
            .await?;
        Ok(maybe_hash.map(|hash| hash.trim_start_matches("0x").to_string()))
    }

    /// Returns the hash of the highest finalized block.
    pub async fn get_finalized_block_hash(&self) -> anyhow::Result<String> {
        let hash: String = self
            .ws_client
            .request("chain_getFinalizedHead", rpc_params!())
            .await?;
        Ok(hash.trim_start_matches("0x").to_string())
    }

    /// Returns the timestamp of the block with the gives hash in milliseconds.
    pub async fn get_block_timestamp(&self, block_hash: &str) -> anyhow::Result<Option<u64>> {
        let maybe_hex_string: Option<String> = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Timestamp", "Now", Some(block_hash))?,
            )
            .await?;
        if let Some(hex_string) = maybe_hex_string {
            Ok(Some(decode_hex_string(hex_string.as_str())?))
        } else {
            Ok(None)
        }
    }

    /// Gets system health data from the node.
    pub async fn get_system_health(&self) -> anyhow::Result<SystemHealth> {
        let system_health: SystemHealth = self
            .ws_client
            .request("system_health", rpc_params!())
            .await?;
        Ok(system_health)
    }

    /// Gets chain properties
    pub async fn get_chain_properties(&self) -> anyhow::Result<ChainProperties> {
        let system_health: ChainProperties = self
            .ws_client
            .request("system_properties", rpc_params!())
            .await?;
        Ok(system_health)
    }

    /// Gets the name of the chain from using the relevant RPC call.
    pub async fn get_chain_name(&self) -> anyhow::Result<String> {
        let name: String = self
            .ws_client
            .request("system_chain", rpc_params!())
            .await?;
        Ok(name)
    }

    /// Gets the header of a block by its hash.
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

    /// Gets execution/storage trace records of a block by its hash.
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

    /// Gets the content (header, extrinsics) of a block by its hash.
    pub async fn get_block(&self, block_hash: &str) -> anyhow::Result<Block> {
        let block_wrapper: BlockWrapper = self
            .ws_client
            .request("chain_getBlock", rpc_params!(block_hash))
            .await?;
        Ok(block_wrapper.block)
    }

    /// Gets the weight bytes of a block by its hash.
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
                tracing::error!("{message}");
                anyhow::anyhow!(message)
            })?;

        loop {
            let next_item = timeout(subscription_timeout, subscription.next());
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    tracing::info!("🚫 Block subscription cancelled.");
                    return Ok(());
                }
                result = next_item => {
                    match result {
                        Ok(Some(Ok(block_header))) => {
                            if let Err(error) = callback(block_header).await {
                                let message = format!("⚠️ Error in callback: {error:?}");
                                tracing::error!("{message}");
                                return Err(anyhow::anyhow!(error)).context(message);
                            }
                        }
                        Ok(Some(Err(error))) => {
                            let message = format!("⚠️ Error while receiving block header: {error:?}");
                            tracing::error!("{message}");
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

    /// Function to subscribe to proposed (i.e. best) blocks though websocket.
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

    /// Function to subscribe to finalized blocks though websocket.
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

    /// Gets the last runtime upgrade info at a block height.
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

    /// Gets encoded runtime metadata hex string at a block.
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

    /// Gets decoded runtime metadata at a block.
    pub async fn get_metadata_at_block(&self, block_hash: &str) -> anyhow::Result<RuntimeMetadata> {
        let metadata_hex_string = self.get_metadata_hex_string_at_block(block_hash).await?;
        let mut metadata_hex_decoded: &[u8] = &hex::decode(metadata_hex_string)?;
        let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_hex_decoded)?;
        Ok(metadata.1)
    }

    /// Gets active validator account ids (could be 32-byte or 20-byte) for chains with the Session pallet.
    pub async fn get_active_validator_account_ids<T: Decode>(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<T>> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "Validators", Some(block_hash))?,
            )
            .await?;
        let account_ids: Vec<T> = decode_hex_string(hex_string.as_str())?;
        Ok(account_ids)
    }

    /// Gets the session index at a given block.
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

    /// Gets the events in a block in encoded bytes.
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

    /// Returns the block author for a given block hash if available.
    /// Currently targeting Moonbeam only.
    /// Returned value is a 20-byte Ethereum-style address.
    ///
    /// # Arguments
    /// * `block_hash` Hash of the block in hexadecimal format, with or without a leading `0x`.
    pub async fn get_nimbus_block_author(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<MultiAddress>> {
        let maybe_address_hex_string: Option<String> = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("AuthorInherent", "Author", Some(block_hash))?,
            )
            .await?;
        if let Some(address_hex_string) = maybe_address_hex_string {
            Ok(Some(MultiAddress::from_str(&address_hex_string)?))
        } else {
            Ok(None)
        }
    }
}

use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use jsonrpsee::tokio::time::timeout;
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee_core::client::{Client, ClientT, Subscription, SubscriptionClientT};
use jsonrpsee_core::rpc_params;
use parity_scale_codec::Decode;
use sp_runtime::AccountId32;
use std::future::Future;
use submerge_base::args::RPCArgs;
use submerge_base::types::substrate::block::{Block, BlockHeader, BlockWrapper};
use submerge_base::types::substrate::block_trace::{BlockTrace, BlockTraceWrapper, StorageMethod};
use submerge_base::types::substrate::runtime::LastRuntimeUpgradeInfo;
use submerge_util::substrate::storage::{decode_hex_string, get_rpc_storage_plain_params};

pub struct SubstrateClient {
    ws_client: Client,
}

impl SubstrateClient {
    pub async fn new(args: &RPCArgs) -> anyhow::Result<Self> {
        Self::new_inner(
            &args.rpc_url,
            args.rpc_connection_timeout_secs,
            args.rpc_request_timeout_secs,
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
                get_rpc_storage_plain_params("Timestamp", "Now", Some(block_hash)),
            )
            .await?;
        decode_hex_string(hex_string.as_str())
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
                get_rpc_storage_plain_params("System", "BlockWeight", Some(block_hash)),
            )
            .await?;
        if let Some(hex_string) = maybe_hex_string {
            Ok(Some(hex::decode(hex_string.trim_start_matches("0x"))?))
        } else {
            Ok(None)
        }
    }

    async fn subscribe_to_blocks<F>(
        &self,
        subscribe_method_name: &str,
        unsubscribe_method_name: &str,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        let mut subscription: Subscription<BlockHeader> = match self
            .ws_client
            .subscribe(
                subscribe_method_name,
                rpc_params!(),
                unsubscribe_method_name,
            )
            .await
        {
            Ok(subscription) => subscription,
            Err(error) => {
                log::error!("⚠️ Error while subscribing to blocks: {error:?}");
                return;
            }
        };

        while let Ok(maybe_block_header_result) = timeout(
            std::time::Duration::from_secs(timeout_seconds),
            subscription.next(),
        )
        .await
        {
            match maybe_block_header_result {
                Some(block_header_result) => match block_header_result {
                    Ok(block_header) => {
                        if let Err(error) = callback(block_header).await {
                            log::error!("⚠️ Error in callback: {error:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        log::error!("⚠️ Error while getting block header: {error:?}");
                        log::error!("Will exit new block subscription.");
                        break;
                    }
                },
                None => {
                    log::error!("⚠️ Empty block header. Will exit new block subscription.");
                    break;
                }
            }
        }
    }

    pub async fn subscribe_to_new_blocks<F>(
        &self,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        self.subscribe_to_blocks(
            "chain_subscribeNewHeads",
            "chain_unsubscribeNewHeads",
            timeout_seconds,
            callback,
        )
        .await;
    }

    /// Subscribes to finalized blocks.
    pub async fn subscribe_to_finalized_blocks<F>(
        &self,
        timeout_seconds: u64,
        callback: impl Fn(BlockHeader) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        self.subscribe_to_blocks(
            "chain_subscribeFinalizedHeads",
            "chain_unsubscribeFinalizedHeads",
            timeout_seconds,
            callback,
        )
        .await;
    }

    pub async fn get_last_runtime_upgrade_info(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<LastRuntimeUpgradeInfo> {
        let upgrade_info: LastRuntimeUpgradeInfo = self
            .ws_client
            .request("state_getRuntimeVersion", rpc_params!(block_hash))
            .await?;
        // let upgrade_info = LastRuntimeUpgradeInfo::from_substrate_hex_string(hex_string)?;
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
    ) -> anyhow::Result<Vec<AccountId32>> {
        let hex_string: String = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "Validators", Some(block_hash)),
            )
            .await?;
        let account_ids: Vec<AccountId32> = decode_hex_string(hex_string.as_str())?;
        Ok(account_ids)
    }

    pub async fn get_current_session_index(&self, block_hash: &str) -> anyhow::Result<u32> {
        let maybe_hex_string: Option<String> = self
            .ws_client
            .request(
                "state_getStorage",
                get_rpc_storage_plain_params("Session", "CurrentIndex", Some(block_hash)),
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
                get_rpc_storage_plain_params("System", "Events", Some(block_hash)),
            )
            .await?;
        Ok(hex::decode(events_hex_string.trim_start_matches("0x"))?)
    }
}

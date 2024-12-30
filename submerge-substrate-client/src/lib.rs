use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee_core::client::{Client, ClientT};
use jsonrpsee_core::rpc_params;
use storage_utility::{decode_hex_string, get_rpc_storage_plain_params};
use submerge_types::substrate::block::BlockHeader;
use submerge_types::substrate::block_trace::{BlockTrace, BlockTraceWrapper, StorageMethod};

mod storage_utility;

pub struct SubstrateClient {
    ws_client: Client,
}

impl SubstrateClient {
    pub async fn new(
        rpc_url: &str,
        connection_timeout: u64,
        request_timeout: u64,
    ) -> anyhow::Result<Self> {
        log::info!("Constructing Substrate client.");
        let ws_client = WsClientBuilder::default()
            .connection_timeout(std::time::Duration::from_secs(connection_timeout))
            .request_timeout(std::time::Duration::from_secs(request_timeout))
            .build(rpc_url)
            .await?;
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
}

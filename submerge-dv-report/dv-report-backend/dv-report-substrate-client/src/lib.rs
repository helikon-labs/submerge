use crate::event::get_referendum_events_in_block;
use crate::storage_utility::{decode_hex_string, get_rpc_storage_plain_params};
use crate::vote::get_vote_calls_in_block;
pub use dv_report_metadata::metadata;
pub use dv_report_metadata::metadata_current::{
    self, api::referenda::storage::types::referendum_info_for::ReferendumInfoFor as ReferendumInfo,
};
use dv_report_types::substrate::block::{Block, BlockHeader};
use dv_report_types::substrate::event::ReferendumEvent;
use dv_report_types::substrate::network::Network;
use dv_report_types::substrate::vote::BlockVoteCalls;
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee_core::client::{Client, ClientT};
use jsonrpsee_core::rpc_params;
use std::str::FromStr;
use std::time::Duration;
use subxt::backend::rpc::reconnecting_rpc_client::{ExponentialBackoff, RpcClient};
use subxt::ext::codec::Decode;
use subxt::metadata::Metadata;
use subxt::utils::H256;
use subxt::{OnlineClient, PolkadotConfig};

mod event;
mod storage_utility;
mod vote;

pub struct SubstrateClient {
    pub network: Network,
    current_api: OnlineClient<PolkadotConfig>,
    api: OnlineClient<PolkadotConfig>,
    ws_client: Client,
}

impl SubstrateClient {
    pub async fn new(
        rpc_url: &str,
        connection_timeout: u64,
        request_timeout: u64,
        metadata_file_path: &Option<String>,
    ) -> anyhow::Result<Self> {
        log::info!("Constructing Substrate client.");
        let connection_timeout = Duration::from_secs(connection_timeout);
        let request_timeout = Duration::from_secs(request_timeout);
        let ws_client = WsClientBuilder::default()
            .connection_timeout(connection_timeout)
            .request_timeout(request_timeout)
            .build(rpc_url)
            .await?;
        let chain: String = ws_client.request("system_chain", rpc_params!()).await?;
        let chain = Network::from_str(chain.as_str())?;
        log::info!("{chain} Substrate connection successful.");

        log::info!("Constructing {} SubXT API.", chain.display);
        let rpc_client_1 = RpcClient::builder()
            .retry_policy(
                ExponentialBackoff::from_millis(100)
                    .max_delay(Duration::from_secs(10))
                    .take(3),
            )
            // There are other configurations as well that can be found at [`reconnecting_rpc_client::ClientBuilder`].
            .request_timeout(request_timeout)
            .connection_timeout(connection_timeout)
            .build(rpc_url)
            .await?;
        let rpc_client_2 = RpcClient::builder()
            .retry_policy(
                ExponentialBackoff::from_millis(100)
                    .max_delay(Duration::from_secs(10))
                    .take(3),
            )
            // There are other configurations as well that can be found at [`reconnecting_rpc_client::ClientBuilder`].
            .request_timeout(request_timeout)
            .connection_timeout(connection_timeout)
            .build(rpc_url)
            .await?;
        let current_api = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client_1).await?;
        let api = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client_2).await?;
        if let Some(metadata_file_path) = metadata_file_path {
            let metadata = {
                let bytes = std::fs::read(metadata_file_path)?;
                Metadata::decode(&mut &*bytes)?
            };
            api.set_metadata(metadata);
        }
        log::info!("SubXT {} API ready.", chain.display);
        Ok(SubstrateClient {
            network: chain,
            current_api,
            api,
            ws_client,
        })
    }

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

    pub async fn get_finalized_block_header(&self) -> anyhow::Result<BlockHeader> {
        let hash = self.get_finalized_block_hash().await?;
        self.get_block_header(&hash).await
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

    pub async fn get_block(&self, hash: &str) -> anyhow::Result<Block> {
        let header = self.get_block_header(hash).await?;
        let timestamp = self.get_block_timestamp(hash).await?;
        Ok(Block {
            timestamp,
            number: header.get_number()?,
            hash: hash.to_string().trim_start_matches("0x").to_string(),
            parent_hash: header.parent_hash,
        })
    }

    pub async fn get_block_by_number(&self, number: u64) -> anyhow::Result<Block> {
        let hash = self.get_block_hash(number).await?;
        self.get_block(&hash).await
    }

    pub async fn get_referendum_count(&self, at: &str) -> anyhow::Result<u32> {
        let storage_query = metadata::api::storage().referenda().referendum_count();
        let block_hash = H256::from_str(at)?;
        let referendum_count = self
            .api
            .storage()
            .at(block_hash)
            .fetch(&storage_query)
            .await?
            .unwrap();
        Ok(referendum_count)
    }

    pub async fn get_referendum_info(
        &self,
        index: u32,
        at: &str,
    ) -> anyhow::Result<Option<ReferendumInfo>> {
        let storage_query = metadata_current::api::storage()
            .referenda()
            .referendum_info_for(index);
        let block_hash = H256::from_str(at)?;
        let maybe_referendum_info = self
            .current_api
            .storage()
            .at(block_hash)
            .fetch(&storage_query)
            .await?;
        Ok(maybe_referendum_info)
    }

    pub async fn get_vote_calls_in_block(
        &self,
        network_id: u32,
        block_hash: &str,
    ) -> anyhow::Result<BlockVoteCalls> {
        let block = self.get_block(block_hash).await?;
        let block_hash = H256::from_str(block_hash)?;
        let substrate_block = self.api.blocks().at(block_hash).await?;
        get_vote_calls_in_block(network_id, &block, &substrate_block).await
    }

    pub async fn get_referendum_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ReferendumEvent>> {
        let block_hash = H256::from_str(block_hash)?;
        let block = self.api.blocks().at(block_hash).await?;
        get_referendum_events_in_block(&block).await
    }

    pub async fn get_extrinsic_hash(
        &self,
        block_hash: &str,
        extrinsic_index: u32,
    ) -> anyhow::Result<String> {
        let block_hash = H256::from_str(block_hash)?;
        let block = self.api.blocks().at(block_hash).await?;
        let extrinsics = block.extrinsics().await?;
        for (index, extrinsic) in extrinsics.iter().enumerate() {
            if (index as u32) == extrinsic_index {
                return Ok(hex::encode(extrinsic.hash().0));
            }
        }
        Err(anyhow::Error::msg("Extrinsic not found."))
    }

    pub fn set_metadata(&self, path: &str) -> anyhow::Result<()> {
        let metadata = {
            let bytes = std::fs::read(path)?;
            Metadata::decode(&mut &*bytes)?
        };
        self.api.set_metadata(metadata);
        Ok(())
    }
}

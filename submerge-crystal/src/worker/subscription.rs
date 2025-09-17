use std::{sync::Arc, time::Duration};

use submerge_base::types::substrate::block::BlockHeader;
use submerge_substrate_client::SubstrateClient;
use submerge_util::string::truncate_hash;

use super::processor::BlockProcessor;
use crate::types::BlockStatus;

impl super::Worker {
    async fn on_block(
        &self,
        processor: Arc<BlockProcessor>,
        header: &BlockHeader,
        block_status: BlockStatus,
        skip_traces: bool,
    ) -> anyhow::Result<()> {
        let hash_bytes = header.get_hash_bytes()?;
        let hash_hex = hex::encode(hash_bytes);
        let number = header.get_number()?;
        let id = self.id.to_string();
        crate::metrics::target_best_block_number()
            .with_label_values(&[&id])
            .set(number as i64);
        log::info!(
            "🟦 New proposed block [{number}][0x{}].",
            truncate_hash(&hash_hex)
        );
        match processor
            .process_block(skip_traces, false, &hash_hex, number, block_status)
            .await
        {
            Ok(_) => {
                crate::metrics::processed_best_block_number()
                    .with_label_values(&[&id])
                    .set(number as i64);
            }
            Err(error) => {
                processor
                    .save_block_error(
                        &hash_bytes,
                        number,
                        BlockStatus::Proposed,
                        &error.to_string(),
                    )
                    .await?;
                return Err(error);
            }
        }
        Ok(())
    }

    pub(super) async fn subscribe_to_blocks(
        &self,
        block_status: BlockStatus,
        skip_traces: bool,
    ) -> anyhow::Result<()> {
        let block_processor = match BlockProcessor::new(
            self.id,
            self.config.postgres.clone(),
            &self.config.rpc_config,
            &self.config.legacy_decode_api_url,
        )
        .await
        {
            Ok(block_processor) => Arc::new(block_processor),
            Err(error) => {
                log::error!("🔴 Error while constructing the block processor for new block subscription: {error:?}");
                return Err(error);
            }
        };
        let substrate_client = match SubstrateClient::new(&self.config.rpc_config).await {
            Ok(substrate_client) => substrate_client,
            Err(error) => {
                log::error!("🔴 Error while constructing the Substrate client for new block subscription: {error:?}");
                return Err(error);
            }
        };

        let subscription_timeout =
            Duration::from_secs(self.config.rpc_config.rpc_subscription_timeout_secs);
        match block_status {
            BlockStatus::Proposed => {
                substrate_client
                    .subscribe_to_new_blocks(
                        subscription_timeout,
                        self.cancellation_token.clone(),
                        |header| {
                            let processor = block_processor.clone();
                            async move {
                                self.on_block(
                                    processor,
                                    &header,
                                    BlockStatus::Proposed,
                                    skip_traces,
                                )
                                .await?;
                                Ok(())
                            }
                        },
                    )
                    .await
            }
            BlockStatus::Finalized => {
                substrate_client
                    .subscribe_to_finalized_blocks(
                        subscription_timeout,
                        self.cancellation_token.clone(),
                        |header| {
                            let processor = block_processor.clone();
                            async move {
                                self.on_block(
                                    processor,
                                    &header,
                                    BlockStatus::Finalized,
                                    skip_traces,
                                )
                                .await?;
                                Ok(())
                            }
                        },
                    )
                    .await
            }
            BlockStatus::Pruned => anyhow::bail!("🔴 Cannot subscribe to pruned blocks."),
        }
    }
}

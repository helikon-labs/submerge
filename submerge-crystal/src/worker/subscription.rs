use std::{sync::Arc, time::Duration};

use submerge_base::types::substrate::block::BlockHeader;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::string::truncate_hash;
use tokio::sync::{Mutex, Semaphore};

use super::processor::BlockProcessor;
use crate::{persistence::CrystalPostgreSQLStorage as _, types::BlockStatus};

async fn on_finalized_block(
    worker_id: String,
    postgres: Arc<PostgreSQLStorage>,
    processor: Arc<BlockProcessor>,
    header: BlockHeader,
    skip_traces: bool,
) -> anyhow::Result<()> {
    let finalized_block_number = header.get_number()?;
    crate::metrics::target_finalized_block_number()?
        .with_label_values(&[&worker_id])
        .set(finalized_block_number as i64);
    tracing::info!("🟦 New finalized block [{finalized_block_number}].");
    let mut start_block_number = finalized_block_number;
    if let Some((last_finalized_number, _)) = postgres
        .get_last_indexed_finalized_block_number_and_hash()
        .await?
    {
        let gap = finalized_block_number.saturating_sub(last_finalized_number);
        if gap > 1 {
            start_block_number = last_finalized_number + 1;
            tracing::info!(
                "🟦 Process finalized block range {start_block_number}-{finalized_block_number}."
            );
        } else {
            tracing::info!("🟦 Process finalized block {finalized_block_number}.");
        }
    }

    for block_number in start_block_number..=finalized_block_number {
        let Some(hash_hex) = processor.get_block_hash_hex(block_number).await? else {
            anyhow::bail!("New finalized block {block_number} not found on the RPC node.");
        };
        let hash_bytes = hex::decode(&hash_hex)?;
        match processor
            .process_block(
                skip_traces,
                false,
                &hash_hex,
                block_number,
                BlockStatus::Finalized,
            )
            .await
        {
            Ok(_) => {
                postgres
                    .set_last_indexed_finalized_block_number_and_hash(block_number, &hash_bytes)
                    .await?;
                crate::metrics::processed_finalized_block_number()?
                    .with_label_values(&[&worker_id])
                    .set(block_number as i64);
            }
            Err(error) => {
                processor
                    .save_block_error(
                        &hash_bytes,
                        block_number,
                        BlockStatus::Finalized,
                        &error.to_string(),
                    )
                    .await?;
                return Err(error);
            }
        }
    }
    Ok(())
}

async fn on_proposed_block(
    worker_id: String,
    processor: Arc<BlockProcessor>,
    header: BlockHeader,
    skip_traces: bool,
) -> anyhow::Result<()> {
    let number = header.get_number()?;
    let Some(hash_hex) = processor.get_block_hash_hex(number).await? else {
        anyhow::bail!("Proposed block {number} not found on the RPC node.");
    };
    let hash_bytes = hex::decode(&hash_hex)?;
    crate::metrics::target_best_block_number()?
        .with_label_values(&[&worker_id])
        .set(number as i64);
    tracing::info!(
        "🟦 New proposed block [{number}][0x{}].",
        truncate_hash(&hash_hex),
    );

    match processor
        .process_block(skip_traces, false, &hash_hex, number, BlockStatus::Proposed)
        .await
    {
        Ok(_) => crate::metrics::processed_best_block_number()?
            .with_label_values(&[&worker_id])
            .set(number as i64),
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

impl super::Worker {
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
                tracing::error!("🔴 Error while constructing the block processor for new block subscription: {error:?}");
                return Err(error);
            }
        };
        let substrate_client = match SubstrateClient::new(&self.config.rpc_config).await {
            Ok(substrate_client) => substrate_client,
            Err(error) => {
                tracing::error!("🔴 Error while constructing the Substrate client for new block subscription: {error:?}");
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
                            let worker_id = self.id.to_string();
                            let processor = block_processor.clone();
                            let header = header.clone();
                            async move {
                                on_proposed_block(worker_id, processor, header, skip_traces)
                                    .await?;
                                Ok(())
                            }
                        },
                    )
                    .await
            }
            BlockStatus::Finalized => {
                let last_error: Arc<Mutex<Option<anyhow::Error>>> = Arc::new(Mutex::new(None));
                let gate = Arc::new(Semaphore::new(1));
                substrate_client
                    .subscribe_to_finalized_blocks(
                        subscription_timeout,
                        self.cancellation_token.clone(),
                        |header| {
                            let gate = gate.clone();
                            let processor = block_processor.clone();
                            let last_error = last_error.clone();
                            async move {
                                if let Some(err) = last_error.lock().await.take() {
                                    tracing::error!("⚠️ Previous finalized block processing failed, returning error.");
                                    return Err(err);
                                }
                                let permit = match gate.try_acquire_owned() {
                                    Ok(p) => p,
                                    Err(_) => {
                                        match header.get_number() {
                                            Ok(n) => tracing::warn!("⚠️ Skipping finalized block {n} - previous still processing."),
                                            Err(e) => tracing::warn!("⚠️ Skipping finalized block (unknown number: {e}) - previous still processing."),
                                        }
                                        return Ok(());
                                    }
                                };
                                let worker_id = self.id.to_string();
                                let postgres = self.config.postgres.clone();
                                let header = header.clone();
                                tokio::spawn(async move {
                                    // hold permit for the entire task duration
                                    let _permit_guard = permit;
                                    let result = on_finalized_block(
                                        worker_id,
                                        postgres,
                                        processor,
                                        header,
                                        skip_traces,
                                    )
                                    .await;
                                    if let Err(e) = result {
                                        tracing::error!("Error processing finalized block: {e:?}");
                                        *last_error.lock().await = Some(e);
                                    }
                                    // _permit_guard drops here, releasing the semaphore
                                });
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

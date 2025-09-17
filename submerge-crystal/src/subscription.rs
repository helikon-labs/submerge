/*
use crate::processor::BlockProcessor;
use crate::types::BlockStatus;
use crate::Crystal;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use submerge_base::args::RPCArgs;
use submerge_substrate_client::SubstrateClient;
use submerge_util::string::truncate_hash;
use tokio::sync::RwLock;
use tokio::time::sleep;

static IS_BUSY: AtomicBool = AtomicBool::new(false);

impl Crystal {
    async fn run_new_block_subscription(
        rpc_args: RPCArgs,
        processor: Arc<BlockProcessor>,
        delay: Duration,
        skip_traces: bool,
    ) {
        loop {
            let substrate_client = match SubstrateClient::new(&rpc_args).await {
                Ok(substrate_client) => substrate_client,
                Err(error) => {
                    log::error!("🔴 Error while constructing Substrate client for new block subscription: {error:?}");
                    log::error!("🔄 Will retry after {} seconds.", delay.as_secs());
                    sleep(delay).await;
                    continue;
                }
            };
            substrate_client
                .subscribe_to_new_blocks(rpc_args.rpc_subscription_timeout_secs, |header| {
                    let processor = processor.clone();
                    async move {
                        let hash_bytes = header.get_hash_bytes()?;
                        let hash_hex = hex::encode(hash_bytes);
                        let number = header.get_number()?;
                        crate::metrics::target_best_block_number().set(number as i64);
                        log::info!(
                            "🟦 New proposed block [{number}][0x{}].",
                            truncate_hash(&hash_hex)
                        );

                        match processor
                            .process_block(skip_traces, &hash_hex, number, BlockStatus::Proposed)
                            .await
                        {
                            Ok(_) => {
                                crate::metrics::processed_best_block_number().set(number as i64)
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
                })
                .await;
            log::error!("🔴 New block subscription exited.");
            log::error!("🔄 Will resubscribe after {} seconds.", delay.as_secs());
            sleep(delay).await;
        }
    }

    async fn run_finalized_block_subscription(
        rpc_args: RPCArgs,
        processor: Arc<BlockProcessor>,
        delay: Duration,
        start_block: u64,
        scan: bool,
        skip_traces: bool,
        stop_on_error: bool,
    ) {
        let error_cell = Arc::new(RwLock::new(OnceCell::new()));
        loop {
            let substrate_client = match SubstrateClient::new(&rpc_args).await {
                Ok(substrate_client) => substrate_client,
                Err(error) => {
                    log::error!("🔴 Error while constructing Substrate client for new block subscription: {error:?}");
                    log::error!("🔄 Will retry after {} seconds.", delay.as_secs());
                    sleep(delay).await;
                    continue;
                }
            };
            substrate_client
                .subscribe_to_finalized_blocks(rpc_args.rpc_subscription_timeout_secs, |header| {
                    let processor = processor.clone();
                    let error_cell = error_cell.clone();
                    async move {
                        {
                            let mut err_cell = error_cell.write().await;
                            if let Some(e) = err_cell.take() {
                                return Err(e);
                            }
                        }

                        let number = header.get_number()?;
                        let block_hash = hex::encode(header.get_hash_bytes()?);
                        crate::metrics::target_finalized_block_number().set(number as i64);

                        if IS_BUSY
                            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                            .is_err()
                        {
                            log::info!(
                                "⏳ Busy processing older finalized blocks. Skipping [{number}][0x{}].",
                                truncate_hash(&block_hash),
                            );
                            return Ok(());
                        }
                        let processor = processor.clone();
                        let error_cell = error_cell.clone();
                        tokio::spawn(async move {
                            let result = processor
                                .process_finalized_blocks_in_range(
                                    scan,
                                    stop_on_error,
                                    skip_traces,
                                    start_block,
                                    number,
                                )
                                .await;
                            if let Err(error) = result {
                                if let Ok(error_cell) = error_cell.try_write() {
                                    let _ = error_cell.set(error); // Handle the Result properly
                                } else {
                                    log::error!(
                                        "🔴 Failed to set error in error cell: lock contention"
                                    );
                                }
                            }
                            IS_BUSY.store(false, Ordering::SeqCst);
                            Ok::<_, anyhow::Error>(())
                        });
                        Ok(())
                    }
                })
                .await;
            log::error!("🔴 Finalized block subscription exited.");
            log::error!("🔄 Will resubscribe after {} seconds.", delay.as_secs());
            sleep(delay).await;
        }
    }

    pub async fn subscribe_to_live_blocks(
        &self,
        processor: Arc<BlockProcessor>,
    ) -> anyhow::Result<()> {
        let delay = Duration::from_secs(self.args.service.recovery_sleep_seconds);
        loop {
            let rpc_args = self.args.rpc.clone();
            let skip_traces = self.args.skip_traces;
            let scan = self.args.scan;
            let stop_on_error = self.args.stop_on_error;
            let start_block = self.args.start_block.unwrap_or(0);

            let proposed_block_processor = processor.clone();
            let finalized_block_processor = processor.clone();

            let proposed = tokio::spawn(Self::run_new_block_subscription(
                rpc_args.clone(),
                proposed_block_processor,
                delay,
                skip_traces,
            ));

            let finalized = tokio::spawn(Self::run_finalized_block_subscription(
                rpc_args,
                finalized_block_processor,
                delay,
                start_block,
                scan,
                skip_traces,
                stop_on_error,
            ));

            let _ = tokio::join!(proposed, finalized);
            log::error!("🔴 All subscriptions exited.");
            log::error!("🔄 Will restart after {delay:?}.");
            sleep(delay).await;
        }
    }
}

*/

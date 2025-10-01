use async_trait::async_trait;
use dv_report_config::Config;
use dv_report_repository::Repository;
use dv_report_service::Service;
use dv_report_types::substrate::event::ReferendumEvent;
use dv_report_types::substrate::network::Network;
use std::cmp::max;
use std::time::Duration;
use tokio::time::sleep;

mod metrics;

#[derive(Debug, Default)]
pub struct Indexer {
    config: Config,
}

async fn process_block(
    repository: &Repository,
    network_id: u32,
    cohort_number: u32,
    block_number: u64,
) -> anyhow::Result<()> {
    log::info!("Process block {block_number}.");
    let block = repository.get_block_by_number(block_number).await?;
    let block_vote_calls = repository
        .get_vote_calls_in_block(network_id, block_number)
        .await?;
    let block_referendum_events = repository
        .get_referendum_events_in_block(block_number)
        .await?;
    let mut new_referenda = Vec::new();
    for block_referendum_event in block_referendum_events.iter() {
        if let ReferendumEvent::Submitted {
            referendum_index,
            track_id: _,
        } = block_referendum_event
        {
            log::info!("New referendum {referendum_index}.");
            let new_referendum = repository
                .get_ongoing_referendum(network_id, *referendum_index, block.hash.as_str())
                .await?;
            new_referenda.push(new_referendum);
        }
    }
    repository
        .save_block_with_details(
            network_id,
            cohort_number,
            &block,
            &new_referenda,
            &block_referendum_events,
            &block_vote_calls,
        )
        .await?;
    Ok(())
}

async fn import_comments(config: &Config) -> anyhow::Result<()> {
    let network = Network::from_id(config.substrate.network_id);
    let repository = Repository::new(config).await?;
    let referenda = repository.get_network_referenda(network.id).await?;
    let mut imported_comment_count = 0;
    for referendum in referenda.iter() {
        log::info!(
            "Import Subsquare comments for {} #{}.",
            network.token_ticker,
            referendum.index
        );
        let subsquare_comments = repository
            .get_subsquare_referendum_comments(&network, referendum.index)
            .await?;
        for subsquare_comment in subsquare_comments.iter() {
            repository
                .save_subsquare_referendum_comment(network.id, referendum.index, subsquare_comment)
                .await?;
        }
        imported_comment_count += subsquare_comments.len();
        log::info!(
            "Import Polkassembly comments for {} #{}.",
            network.token_ticker,
            referendum.index
        );
        let polkassembly_comments = repository
            .get_polkassembly_referendum_comments(&network, referendum.index)
            .await?;
        for polkassembly_comment in polkassembly_comments.iter() {
            repository
                .save_polkassembly_referendum_comment(
                    network.id,
                    referendum.index,
                    polkassembly_comment,
                    None,
                )
                .await?;
        }
        imported_comment_count += polkassembly_comments.len();
    }
    metrics::imported_comment_count().set(imported_comment_count as i64);
    Ok(())
}

#[async_trait(?Send)]
impl Service for Indexer {
    fn name(&self) -> String {
        format!("{} Indexer", self.config.substrate.chain_display)
    }

    fn get_metrics_server_addr(&self) -> (String, u16) {
        (
            self.config.metrics.host.clone(),
            self.config.metrics.indexer_port,
        )
    }

    async fn run(&self) -> anyhow::Result<()> {
        let repository = Repository::new(&self.config).await?;
        let network = Network::from_id(self.config.substrate.network_id);
        let cohort = repository
            .get_cohort(network.id, self.config.indexer.cohort_number)
            .await?;
        log::info!(
            "{} indexer started for DV Cohort #{}.",
            network.display,
            cohort.number,
        );
        tokio::spawn({
            let config = self.config.clone();
            async move {
                loop {
                    match import_comments(&config).await {
                        Ok(()) => log::info!("Comments imported successfully."),
                        Err(error) => log::error!("Error while importing comments: {}", error),
                    }
                    log::info!("Comment will be imported again after {} minutes.", 30);
                    sleep(Duration::from_secs(30 * 60)).await;
                }
            }
        });
        /* // cohort init cancelled
        let delegates = repository
            .get_cohort_delegates(network.id, cohort.number)
            .await?;
        repository
            .init_cohort(&network, &cohort, delegates.as_slice())
            .await?;
         */
        let delay_seconds = self.config.common.recovery_retry_seconds;
        if let Some(end_block_number) = self.config.indexer.end_block_number {
            let max_block_number = repository.get_max_block_number(network.id).await?;
            let start_block_number = max((max_block_number + 1) as u64, cohort.start_block.number);
            for block_number in start_block_number..=end_block_number {
                process_block(
                    &repository,
                    network.id,
                    self.config.indexer.cohort_number,
                    block_number,
                )
                .await?;
                metrics::indexed_finalized_block_number().set(block_number as i64);
                log::info!("Indexed block {block_number}.");
            }
            return Ok(());
        }
        loop {
            let finalized_block = repository.get_finalized_block().await?;
            let max_block_number = repository.get_max_block_number(network.id).await?;
            let start_block_number = max((max_block_number + 1) as u64, cohort.start_block.number);
            for block_number in start_block_number..=finalized_block.number {
                process_block(
                    &repository,
                    network.id,
                    self.config.indexer.cohort_number,
                    block_number,
                )
                .await?;
                metrics::indexed_finalized_block_number().set(block_number as i64);
                log::info!("Indexed block {block_number}.");
            }
            log::info!(
                "Reached finalized head {}. Will check again in {delay_seconds} seconds.",
                finalized_block.number,
            );
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }
}

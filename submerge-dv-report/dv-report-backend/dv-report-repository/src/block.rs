use crate::Repository;
use dv_report_types::governance::referendum::{Referendum, ReferendumStatus};
use dv_report_types::substrate::block::Block;
use dv_report_types::substrate::event::ReferendumEvent;
use dv_report_types::substrate::vote::BlockVoteCalls;

impl Repository {
    pub async fn get_finalized_block(&self) -> anyhow::Result<Block> {
        let hash = self.substrate_client.get_finalized_block_hash().await?;
        self.substrate_client.get_block(&hash).await
    }

    pub async fn get_vote_calls_in_block(
        &self,
        network_id: u32,
        block_number: u64,
    ) -> anyhow::Result<BlockVoteCalls> {
        let hash = self.substrate_client.get_block_hash(block_number).await?;
        self.substrate_client
            .get_vote_calls_in_block(network_id, hash.as_str())
            .await
    }

    pub async fn get_referendum_events_in_block(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<ReferendumEvent>> {
        let hash = self.substrate_client.get_block_hash(block_number).await?;
        self.substrate_client
            .get_referendum_events_in_block(hash.as_str())
            .await
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> anyhow::Result<Block> {
        self.substrate_client
            .get_block_by_number(block_number)
            .await
    }

    pub async fn save_block(&self, network_id: u32, block: &Block) -> anyhow::Result<()> {
        let mut tx = self.postgres.begin_tx().await?;
        self.postgres.save_block(network_id, block, &mut tx).await?;
        self.postgres.commit_tx(tx).await
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn save_block_with_details(
        &self,
        network_id: u32,
        cohort_number: u32,
        block: &Block,
        new_referenda: &[Referendum],
        referendum_events: &[ReferendumEvent],
        block_vote_calls: &BlockVoteCalls,
    ) -> anyhow::Result<()> {
        let mut tx = self.postgres.begin_tx().await?;
        // save block
        self.postgres.save_block(network_id, block, &mut tx).await?;
        // save referenda
        for referendum in new_referenda.iter() {
            self.postgres
                .save_referendum(referendum, cohort_number, &mut tx)
                .await?;
        }
        // save vote calls
        for vote_call in block_vote_calls.vote_calls.iter() {
            if self
                .postgres
                .referendum_exists(network_id, vote_call.referendum_index, &mut tx)
                .await?
            {
                self.postgres.save_vote_call(vote_call, &mut tx).await?;
            } else {
                log::warn!(
                    "Referendum {} does not exist in the database. Skip vote call.",
                    vote_call.referendum_index,
                );
            }
        }
        // save remove vote calls
        for remove_vote_call in block_vote_calls.remove_vote_calls.iter() {
            if self
                .postgres
                .referendum_exists(network_id, remove_vote_call.referendum_index, &mut tx)
                .await?
            {
                self.postgres
                    .save_remove_vote_call(remove_vote_call, &mut tx)
                    .await?;
            } else {
                log::warn!(
                    "Referendum {} does not exist in the database. Skip remove vote call.",
                    remove_vote_call.referendum_index,
                );
            }
        }
        // save referendum events
        for referendum_event in referendum_events.iter() {
            match referendum_event {
                ReferendumEvent::Submitted {
                    referendum_index,
                    track_id,
                } => {
                    let id = self
                        .postgres
                        .save_referendum_submitted_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            *track_id as u32,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} submitted event. Event database id: #{id}");
                }
                ReferendumEvent::DecisionDepositPlaced {
                    referendum_index,
                    amount,
                    who,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip decision deposit placed event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_decision_deposit_placed_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            *amount,
                            who,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} decision deposit placed event. Event database id: #{id}");
                }
                ReferendumEvent::DecisionDepositRefunded {
                    referendum_index,
                    amount,
                    who,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip decision deposit refunded event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_decision_deposit_refunded_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            *amount,
                            who,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} decision deposit refunded event. Event database id: #{id}");
                }
                ReferendumEvent::DepositSlashed { amount, who } => {
                    let id = self
                        .postgres
                        .save_referendum_deposit_slashed_event(
                            network_id,
                            block.hash.as_str(),
                            *amount,
                            who,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum deposit slashed event. Event database id: #{id}");
                }
                ReferendumEvent::DecisionStarted {
                    referendum_index,
                    track_id,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip decision started event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_decision_started_event(
                            network_id,
                            block.hash.as_str(),
                            *track_id,
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} decision started event. Event database id: #{id}");
                }
                ReferendumEvent::ConfirmStarted { referendum_index } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip confirm started event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_confirm_started_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} confirm started event. Event database id: #{id}");
                }
                ReferendumEvent::ConfirmAborted { referendum_index } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip confirm aborted event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_confirm_aborted_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} confirm aborted event. Event database id: #{id}");
                }
                ReferendumEvent::Confirmed {
                    referendum_index,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip confirmed event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::Confirmed,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_confirmed_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} confirmed event. Event database id: #{id}");
                }
                ReferendumEvent::Approved { referendum_index } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip approved event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::Confirmed,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_approved_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} approved event. Event database id: #{id}");
                }
                ReferendumEvent::Rejected {
                    referendum_index,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip rejected event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::Rejected,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_rejected_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} rejected event. Event database id: #{id}");
                }
                ReferendumEvent::Cancelled {
                    referendum_index,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip cancelled event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::Cancelled,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_cancelled_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} cancelled event. Event database id: #{id}");
                }
                ReferendumEvent::TimedOut {
                    referendum_index,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip timed out event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::TimedOut,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_timed_out_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} timed out event. Event database id: #{id}");
                }
                ReferendumEvent::Killed {
                    referendum_index,
                    tally,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip killed event.");
                        continue;
                    }
                    self.postgres
                        .update_referendum_status(
                            network_id,
                            *referendum_index,
                            ReferendumStatus::Killed,
                        )
                        .await?;
                    let id = self
                        .postgres
                        .save_referendum_killed_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            tally,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} killed event. Event database id: #{id}");
                }
                ReferendumEvent::SubmissionDepositRefunded {
                    referendum_index,
                    amount,
                    who,
                } => {
                    if !self
                        .postgres
                        .referendum_exists(network_id, *referendum_index, &mut tx)
                        .await?
                    {
                        log::info!("Referendum {referendum_index} does not exist in the database. Skip submission deposit refunded event.");
                        continue;
                    }
                    let id = self
                        .postgres
                        .save_referendum_submission_deposit_refunded_event(
                            network_id,
                            block.hash.as_str(),
                            *referendum_index,
                            *amount,
                            who,
                            &mut tx,
                        )
                        .await?;
                    log::info!("Saved referendum {referendum_index} submission deposit refunded event. Event database id: #{id}");
                }
            }
        }
        self.postgres.commit_tx(tx).await
    }

    pub async fn get_max_block_number(&self, network_id: u32) -> anyhow::Result<i64> {
        self.postgres.get_max_block_number(network_id).await
    }
}

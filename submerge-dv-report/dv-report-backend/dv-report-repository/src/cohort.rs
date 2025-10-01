use crate::Repository;
use dv_report_substrate_client::ReferendumInfo;
use dv_report_types::dv::cohort::Cohort;
use dv_report_types::dv::delegate::Delegate;
use dv_report_types::governance::referendum::{Referendum, ReferendumStatus};
use dv_report_types::substrate::network::Network;
use dv_report_types::substrate::track::Track;
use std::cmp::Reverse;

impl Repository {
    pub async fn get_all_cohorts(&self) -> anyhow::Result<Vec<Cohort>> {
        let rows = self.postgres.get_all_cohorts().await?;
        let mut cohorts = Vec::new();
        for row in rows.iter() {
            let cohort_start_block = self
                .substrate_client
                .get_block(row.start_block_hash.as_str())
                .await?;
            cohorts.push(row.clone().into_cohort(cohort_start_block));
        }
        Ok(cohorts)
    }

    pub async fn get_cohort(&self, network_id: u32, cohort_number: u32) -> anyhow::Result<Cohort> {
        let cohort_row = self.postgres.get_cohort(network_id, cohort_number).await?;
        let cohort_start_block = self
            .substrate_client
            .get_block(cohort_row.start_block_hash.as_str())
            .await?;
        Ok(cohort_row.into_cohort(cohort_start_block))
    }

    pub async fn get_cohort_delegates(
        &self,
        network_id: u32,
        cohort_number: u32,
    ) -> anyhow::Result<Vec<Delegate>> {
        let delegate_rows = self.postgres.get_all_delegates().await?;
        let mut delegates = Vec::new();
        for delegate_row in delegate_rows {
            let delegation_row = self
                .postgres
                .get_network_cohort_delegation_for_delegate(
                    network_id,
                    cohort_number,
                    delegate_row.id.as_str(),
                )
                .await?;
            let start_block = self
                .substrate_client
                .get_block(delegation_row.start_block_hash.as_str())
                .await?;
            let end_block = if let Some(end_block_hash) = delegation_row.end_block_hash.as_ref() {
                Some(
                    self.substrate_client
                        .get_block(end_block_hash.as_str())
                        .await?,
                )
            } else {
                None
            };
            let delegation = delegation_row.into_delegation(start_block, end_block)?;
            delegates.push(delegate_row.into_delegate(vec![delegation]));
        }
        Ok(delegates)
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn init_cohort(
        &self,
        network: &Network,
        cohort: &Cohort,
        delegates: &[Delegate],
    ) -> anyhow::Result<()> {
        if self.postgres.get_referendum_count(network.id).await? > 0 {
            log::info!("Cohort had been initialized.");
            return Ok(());
        }
        log::info!("Initialize {} cohort #{}.", network.display, cohort.number);
        let referendum_count = self
            .substrate_client
            .get_referendum_count(cohort.start_block.hash.as_str())
            .await?;
        log::info!("{referendum_count} ongoing referenda.");
        let mut tx = self.postgres.begin_tx().await?;
        for referendum_index in 0..referendum_count {
            if let Some(referendum_info) = self
                .substrate_client
                .get_referendum_info(referendum_index, cohort.start_block.hash.as_str())
                .await?
            {
                match referendum_info {
                    ReferendumInfo::Ongoing(status) => {
                        let submission_block_hash = self
                            .substrate_client
                            .get_block_hash(status.submitted as u64)
                            .await?;
                        let submission_block = self
                            .substrate_client
                            .get_block(submission_block_hash.as_str())
                            .await?;
                        self.postgres
                            .save_block(network.id, &submission_block, &mut tx)
                            .await?;
                        let referendum = Referendum {
                            network_id: network.id,
                            index: referendum_index,
                            track: Track::from_id(status.track),
                            submission_block,
                            status: ReferendumStatus::Ongoing,
                        };
                        log::info!(
                            "Save ongoing referendum #{referendum_index} on track {}.",
                            referendum.track.name()
                        );
                        self.postgres
                            .save_referendum(&referendum, cohort.number, &mut tx)
                            .await?;
                        let mut vote_calls = self
                            .subsquare_client
                            .fetch_vote_calls(network, referendum_index)
                            .await?;
                        vote_calls.sort_by_key(|c| Reverse(c.extrinsic.block_number));
                        for delegate in delegates.iter() {
                            let Some(delegation) = delegate
                                .delegations
                                .iter()
                                .find(|d| d.network_id == network.id)
                            else {
                                continue;
                            };
                            if let Some(delegate_vote_call) = vote_calls
                                .iter()
                                .find(|v| v.voter == delegation.delegate_account_id)
                            {
                                if delegate_vote_call.extrinsic.block_number
                                    < cohort.start_block.number
                                {
                                    log::info!(
                                        "{} pre-voted on {}.",
                                        delegate.name,
                                        referendum_index
                                    );
                                    let block = self
                                        .substrate_client
                                        .get_block(delegate_vote_call.extrinsic.block_hash.as_str())
                                        .await?;
                                    self.postgres
                                        .save_block(network.id, &block, &mut tx)
                                        .await?;
                                    self.postgres
                                        .save_referendum(&referendum, cohort.number, &mut tx)
                                        .await?;
                                    let block_vote_calls = self
                                        .substrate_client
                                        .get_vote_calls_in_block(network.id, block.hash.as_str())
                                        .await?;
                                    let vote_call = block_vote_calls
                                        .vote_calls
                                        .iter()
                                        .find(|v| {
                                            v.voter == delegation.delegate_account_id
                                                && v.referendum_index == referendum_index
                                        })
                                        .unwrap();
                                    self.postgres.save_vote_call(vote_call, &mut tx).await?;
                                }
                            }
                        }
                    }
                    _ => log::info!("Skip referendum #{referendum_index}."),
                }
            }
        }
        self.postgres.commit_tx(tx).await?;
        Ok(())
    }
}

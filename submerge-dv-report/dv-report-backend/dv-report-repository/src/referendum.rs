use crate::Repository;
use async_recursion::async_recursion;
use dv_report_substrate_client::ReferendumInfo;
use dv_report_types::governance::polkassembly::PolkassemblyReferendumComment;
use dv_report_types::governance::referendum::{Referendum, ReferendumStatus};
use dv_report_types::governance::subsquare::SubsquareReferendumComment;
use dv_report_types::substrate::network::Network;
use dv_report_types::substrate::track::Track;

impl Repository {
    pub async fn get_referendum_count(&self, block_hash: &str) -> anyhow::Result<u32> {
        self.substrate_client.get_referendum_count(block_hash).await
    }

    pub async fn get_ongoing_referendum(
        &self,
        network_id: u32,
        referendum_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<Referendum> {
        let Some(referendum_info) = self
            .substrate_client
            .get_referendum_info(referendum_index, block_hash)
            .await?
        else {
            return Err(anyhow::Error::msg(format!(
                "Referendum {referendum_index} not found in Substrate storage."
            )));
        };
        let referendum = match referendum_info {
            ReferendumInfo::Ongoing(status) => {
                let submission_block_hash = self
                    .substrate_client
                    .get_block_hash(status.submitted as u64)
                    .await?;
                let submission_block = self
                    .substrate_client
                    .get_block(submission_block_hash.as_str())
                    .await?;
                Referendum {
                    network_id,
                    index: referendum_index,
                    track: Track::from_id(status.track),
                    submission_block,
                    status: ReferendumStatus::Ongoing,
                }
            }
            _ => {
                return Err(anyhow::Error::msg(format!(
                    "Referendum {referendum_index} is not ongoing."
                )))
            }
        };
        Ok(referendum)
    }

    pub async fn get_network_referenda(&self, network_id: u32) -> anyhow::Result<Vec<Referendum>> {
        let mut referenda = Vec::new();
        let referendum_rows = self.postgres.get_network_referenda(network_id).await?;
        for referendum_row in referendum_rows.iter() {
            referenda.push(Referendum {
                network_id,
                index: referendum_row.index as u32,
                track: Track::from_id(referendum_row.track_id as u16),
                submission_block: self
                    .postgres
                    .get_block(
                        referendum_row.network_id as u32,
                        referendum_row.submission_block_hash.as_str(),
                    )
                    .await?,
                status: ReferendumStatus::from_id(referendum_row.status_id as u32),
            });
        }
        Ok(referenda)
    }

    pub async fn get_subsquare_referendum_comments(
        &self,
        chain: &Network,
        referendum_index: u32,
    ) -> anyhow::Result<Vec<SubsquareReferendumComment>> {
        self.subsquare_client
            .fetch_subsquare_referendum_comments(chain, referendum_index)
            .await
    }

    #[async_recursion]
    pub async fn save_subsquare_referendum_comment(
        &self,
        network_id: u32,
        referendum_index: u32,
        comment: &SubsquareReferendumComment,
    ) -> anyhow::Result<()> {
        self.postgres
            .save_subsquare_referendum_comment(network_id, referendum_index, comment)
            .await?;
        if let Some(replies) = comment.replies.as_ref() {
            for reply in replies.iter() {
                self.save_subsquare_referendum_comment(network_id, referendum_index, reply)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_polkassembly_referendum_comments(
        &self,
        chain: &Network,
        referendum_index: u32,
    ) -> anyhow::Result<Vec<PolkassemblyReferendumComment>> {
        self.subsquare_client
            .fetch_polkassembly_referendum_comments(chain, referendum_index)
            .await
    }

    #[async_recursion]
    pub async fn save_polkassembly_referendum_comment(
        &self,
        network_id: u32,
        referendum_index: u32,
        comment: &PolkassemblyReferendumComment,
        reply_to_comment_id: Option<&str>,
    ) -> anyhow::Result<()> {
        self.postgres
            .save_polkassembly_referendum_comment(
                network_id,
                referendum_index,
                comment,
                reply_to_comment_id,
            )
            .await?;
        for reply in comment.replies.iter() {
            self.save_polkassembly_referendum_comment(
                network_id,
                referendum_index,
                reply,
                Some(&comment.id),
            )
            .await?;
        }
        Ok(())
    }
}

use crate::substrate::account_id::AccountId;
use crate::util::string_or_number_to_string;
use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum SubsquareReferendumStatus {
    Confirming,
    Deciding,
    Queueing,
    Preparing,
    Submitted,
    Approved,
    Cancelled,
    Killed,
    TimedOut,
    Rejected,
    Executed,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareBlock {
    #[serde(rename = "blockHeight")]
    pub number: u64,
    #[serde(rename = "blockHash")]
    pub hash: String,
    #[serde(rename = "blockTime")]
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareExtrinsic {
    #[serde(rename = "blockHeight")]
    pub block_number: u64,
    pub block_hash: String,
    #[serde(rename = "blockTime")]
    pub block_timestamp: u64,
    pub extrinsic_index: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareAssetKind {
    pub chain: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareBeneficiary {
    pub chain: String,
    pub address: String,
    #[serde(rename = "pubKey")]
    pub public_key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumSummary {
    pub summary: String,
    #[serde(rename = "indexer")]
    pub block: SubsquareBlock,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumState {
    #[serde(rename = "name")]
    pub status: SubsquareReferendumStatus,
    #[serde(rename = "indexer")]
    pub block: SubsquareBlock,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumContentSummary {
    pub summary: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumLocalSpend {
    pub is_spend_local: bool,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub symbol: String,
    pub amount: String,
    pub beneficiary: AccountId,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumNonLocalSpend {
    pub is_spend_local: bool,
    pub asset_kind: SubsquareAssetKind,
    pub amount: String,
    pub beneficiary: SubsquareBeneficiary,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum SubSquareReferendumSpend {
    NonLocalSpend(SubsquareReferendumNonLocalSpend),
    LocalSpend(SubsquareReferendumLocalSpend),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareTrackInfo {
    pub id: u16,
    pub name: String,
    pub original_name: Option<String>,
    pub max_deciding: u32,
    pub decision_deposit: String,
    pub prepare_period: u32,
    pub decision_period: u32,
    pub confirm_period: u32,
    pub min_enactment_period: u32,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumOnChainDecisionInfo {
    #[serde(rename = "since")]
    pub decision_start_block_number: Option<u64>,
    #[serde(rename = "confirming")]
    pub confirm_start_block_number: Option<u64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumOnChainInfo {
    #[serde(rename = "submitted")]
    pub submission_block_number: u64,
    #[serde(rename = "deciding")]
    pub decision_info: Option<SubsquareReferendumOnChainDecisionInfo>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumOnChainData {
    pub info: SubsquareReferendumOnChainInfo,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendum {
    #[serde(rename = "_id")]
    pub id: String,
    pub referendum_index: u32,
    #[serde(rename = "indexer")]
    pub extrinsic: SubsquareExtrinsic,
    pub proposer: AccountId,
    pub onchain_data: SubsquareReferendumOnChainData,
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: String,
    #[serde(rename = "track")]
    pub track_id: u16,
    pub state: SubsquareReferendumState,
    #[serde(rename = "edited")]
    pub is_edited: Option<bool>,
    pub content_summary: Option<SubsquareReferendumContentSummary>,
    pub all_spends: Option<Vec<SubSquareReferendumSpend>>,
    pub track_info: SubsquareTrackInfo,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquarePagedData<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubsquareVote {
    Standard(SubsquareStandardVote),
    Split(SubsquareSplitVote),
    SplitAbstain(SubsquareSplitAbstainVote),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareStandardVoteInner {
    pub is_aye: bool,
    pub conviction: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareStandardVote {
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub balance: String,
    pub vote: SubsquareStandardVoteInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareSplitVote {
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub aye: String,
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub nay: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareSplitAbstainVote {
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub aye: String,
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub nay: String,
    #[serde(deserialize_with = "string_or_number_to_string")]
    pub abstain: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareVoteCall {
    #[serde(rename = "_id")]
    pub id: String,
    pub referendum_index: u32,
    pub voter: AccountId,
    #[serde(rename = "indexer")]
    pub extrinsic: SubsquareExtrinsic,
    pub is_standard: bool,
    pub is_split: bool,
    pub is_split_abstain: bool,
    vote: serde_json::Value,
}

impl SubsquareVoteCall {
    pub fn get_vote(&self) -> anyhow::Result<SubsquareVote> {
        let vote = if self.is_standard {
            SubsquareVote::Standard(serde_json::from_value(self.vote.clone())?)
        } else if self.is_split {
            SubsquareVote::Split(serde_json::from_value(self.vote.clone())?)
        } else {
            SubsquareVote::SplitAbstain(serde_json::from_value(self.vote.clone())?)
        };
        Ok(vote)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumCommentAuthor {
    pub username: String,
    pub public_key: Option<AccountId>,
    pub address: AccountId,
    pub email_md5: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumCommentReaction {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "comment")]
    pub comment_id: String,
    pub data_source: String,
    pub proposer: AccountId,
    pub cid: String,
    pub created_at: DateTime<Utc>,
    pub parent_cid: String,
    pub reaction: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsquareReferendumComment {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "referendaReferendum")]
    pub referendum_post_id: String,
    #[serde(rename = "replyToComment")]
    pub reply_to_comment_id: Option<String>,
    pub content: String,
    pub content_type: String,
    pub content_version: String,
    pub author: SubsquareReferendumCommentAuthor,
    pub height: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data_source: String,
    pub cid: String,
    pub proposer: AccountId,
    pub replies: Option<Vec<SubsquareReferendumComment>>,
}

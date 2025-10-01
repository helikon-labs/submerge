use crate::governance::vote::AccountVote;
use crate::substrate::account_id::AccountId;
use crate::substrate::block::Block;
use frame_support::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Default)]
pub struct BlockVoteCalls {
    pub vote_calls: Vec<VoteCall>,
    pub remove_vote_calls: Vec<RemoveVoteCall>,
}

impl BlockVoteCalls {
    pub fn append(&mut self, block_vote_calls: &mut BlockVoteCalls) {
        self.vote_calls.append(&mut block_vote_calls.vote_calls);
        self.remove_vote_calls
            .append(&mut block_vote_calls.remove_vote_calls);
    }
}

#[derive(Clone, Debug)]
pub struct VoteCall {
    pub network_id: u32,
    pub block: Block,
    pub extrinsic_index: u32,
    pub extrinsic_hash: String,
    pub is_batch: bool,
    pub is_multisig: bool,
    pub is_multisig_executed: bool,
    pub is_proxy: bool,
    pub is_successful: bool,
    pub signer: AccountId,
    pub voter: AccountId,
    pub referendum_index: u32,
    pub vote: AccountVote<u128>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct VoteCallRow {
    pub id: i32,
    pub network_id: i32,
    pub referendum_index: i32,
    pub block_hash: String,
    pub extrinsic_index: i32,
    pub extrinsic_hash: String,
    pub is_batch: bool,
    pub is_multisig: bool,
    pub is_multisig_executed: bool,
    pub is_proxy: bool,
    pub is_successful: bool,
    pub signer_account_id: String,
    pub voter_account_id: String,
    pub vote_type: String,
    pub is_aye: Option<bool>,
    pub conviction: Option<i32>,
    pub balance: Option<String>,
    pub aye: Option<String>,
    pub nay: Option<String>,
    pub abstain: Option<String>,
    pub subsquare_comment_id: Option<String>,
    pub polkassembly_comment_id: Option<String>,
}

#[derive(Debug)]
pub struct RemoveVoteCall {
    pub network_id: u32,
    pub block: Block,
    pub extrinsic_index: u32,
    pub extrinsic_hash: String,
    pub is_batch: bool,
    pub is_multisig: bool,
    pub is_multisig_executed: bool,
    pub is_proxy: bool,
    pub is_successful: bool,
    pub signer: AccountId,
    pub voter: AccountId,
    pub referendum_index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tally {
    pub ayes: u128,
    pub nays: u128,
    pub support: u128,
}

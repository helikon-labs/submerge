use crate::substrate::account_id::AccountId;
use crate::substrate::block::Block;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub id: u32,
    pub cohort_number: u32,
    pub network_id: u32,
    pub delegator_account_id: AccountId,
    pub delegate_id: String,
    pub delegate_account_id: AccountId,
    pub start_block: Block,
    pub start_extrinsic_hash: String,
    pub start_extrinsic_index: u32,
    pub end_block: Option<Block>,
    pub end_extrinsic_hash: Option<String>,
    pub end_extrinsic_index: Option<u32>,
}

#[derive(Clone, Debug, FromRow)]
pub struct DelegationRow {
    pub id: i32,
    pub cohort_number: i32,
    pub network_id: i32,
    pub delegator_account_id: String,
    pub delegate_id: String,
    pub delegate_account_id: String,
    pub start_block_hash: String,
    pub start_extrinsic_hash: String,
    pub start_extrinsic_index: i32,
    pub end_block_hash: Option<String>,
    pub end_extrinsic_hash: Option<String>,
    pub end_extrinsic_index: Option<i32>,
}

impl DelegationRow {
    pub fn into_delegation(
        self,
        start_block: Block,
        end_block: Option<Block>,
    ) -> anyhow::Result<Delegation> {
        Ok(Delegation {
            id: self.id as u32,
            cohort_number: self.cohort_number as u32,
            network_id: self.network_id as u32,
            delegator_account_id: AccountId::from_str(&self.delegator_account_id)?,
            delegate_id: self.delegate_id.clone(),
            delegate_account_id: AccountId::from_str(&self.delegate_account_id)?,
            start_block,
            start_extrinsic_hash: self.start_extrinsic_hash.clone(),
            start_extrinsic_index: self.start_extrinsic_index as u32,
            end_block,
            end_extrinsic_hash: self.end_extrinsic_hash.clone(),
            end_extrinsic_index: self.end_extrinsic_index.map(|x| x as u32),
        })
    }
}

use dv_report_metadata::metadata::api::multisig::events::MultisigExecuted;
use dv_report_metadata::metadata::api::{
    conviction_voting::calls::types::RemoveVote as RemoveVoteExtrinsic,
    conviction_voting::calls::types::Vote as VoteExtrinsic,
    multisig::calls::types::AsMulti as AsMultiExtrinsic,
    multisig::calls::types::AsMultiThreshold1 as AsMultiThreshold1Extrinsic,
    proxy::calls::types::Proxy as ProxyExtrinsic,
    proxy::calls::types::ProxyAnnounced as ProxyAnnouncedExtrinsic,
    runtime_types::pallet_conviction_voting::pallet::Call as ConvictionVotingCall,
    runtime_types::pallet_multisig::pallet::Call as MultisigCall,
    runtime_types::pallet_proxy::pallet::Call as ProxyCall,
    runtime_types::pallet_utility::pallet::Call as UtilityCall,
    utility::calls::types::Batch as BatchExtrinsic,
    utility::calls::types::BatchAll as BatchAllExtrinsic,
    utility::calls::types::ForceBatch as ForceBatchExtrinsic,
};
use dv_report_metadata::RuntimeCall;
use dv_report_types::substrate::account_id::AccountId;
use dv_report_types::substrate::block::Block;
use dv_report_types::substrate::vote::{BlockVoteCalls, RemoveVoteCall, VoteCall};
use parity_scale_codec::{Decode, Encode};
use subxt::blocks::ExtrinsicEvents;
use subxt::utils::{AccountId32, MultiAddress};
use subxt::{OnlineClient, PolkadotConfig};

pub(crate) type SubstrateBlock = subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>;

fn has_extrinsic_success_event(events: &ExtrinsicEvents<PolkadotConfig>) -> anyhow::Result<bool> {
    let mut is_successful = false;
    for event in events.iter() {
        let event = event?;
        if event.variant_name() == "ExtrinsicSuccess" {
            is_successful = true;
            break;
        }
    }
    Ok(is_successful)
}

fn has_multisig_executed_event(
    events: &ExtrinsicEvents<PolkadotConfig>,
    call_hash: &[u8; 32],
) -> anyhow::Result<bool> {
    for event in events.find::<MultisigExecuted>() {
        let event = event?;
        if event.call_hash == *call_hash {
            return Ok(true);
        }
    }
    Ok(false)
}

#[allow(clippy::too_many_arguments)]
fn get_vote_call_in_conviction_voting_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    is_batch: bool,
    is_multisig: bool,
    is_multisig_executed: bool,
    is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    voter: &AccountId,
    call: &ConvictionVotingCall,
) -> anyhow::Result<Option<VoteCall>> {
    log::trace!("Inspect conviction voting call for a vote call.");
    let maybe_vote_call = match call {
        ConvictionVotingCall::vote { poll_index, vote } => {
            log::trace!("Vote call found : {poll_index}, vote: {vote:?}");
            Some(VoteCall {
                network_id,
                block: block.clone(),
                extrinsic_index,
                extrinsic_hash: extrinsic_hash.to_string(),
                is_batch,
                is_multisig,
                is_multisig_executed,
                is_proxy,
                is_successful,
                signer: *signer,
                voter: *voter,
                referendum_index: *poll_index,
                vote: vote.into(),
            })
        }
        _ => None,
    };
    Ok(maybe_vote_call)
}

#[allow(clippy::too_many_arguments)]
fn get_remove_vote_call_in_conviction_voting_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    is_batch: bool,
    is_multisig: bool,
    is_multisig_executed: bool,
    is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    voter: &AccountId,
    call: &ConvictionVotingCall,
) -> anyhow::Result<Option<RemoveVoteCall>> {
    log::trace!("Inspect conviction voting call for a remove vote call.");
    let maybe_remove_vote_call = match call {
        ConvictionVotingCall::vote { .. } => None,
        ConvictionVotingCall::remove_vote { class: _, index } => Some(RemoveVoteCall {
            network_id,
            block: block.clone(),
            extrinsic_index,
            extrinsic_hash: extrinsic_hash.to_string(),
            is_batch,
            is_multisig,
            is_multisig_executed,
            is_proxy,
            is_successful,
            signer: *signer,
            voter: *voter,
            referendum_index: *index,
        }),
        _ => None,
    };
    Ok(maybe_remove_vote_call)
}

#[allow(clippy::too_many_arguments)]
fn get_vote_calls_in_proxy_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    events: &ExtrinsicEvents<PolkadotConfig>,
    is_batch: bool,
    is_multisig: bool,
    is_multisig_executed: bool,
    _is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    _voter: &AccountId,
    call: &ProxyCall,
) -> anyhow::Result<BlockVoteCalls> {
    log::trace!("Inspect proxy call.");
    let vote_calls = match call {
        ProxyCall::proxy {
            real,
            force_proxy_type: _,
            call,
        } => {
            let Some(real) = get_account_id_from_multi(real)? else {
                log::warn!("Proxy real account is not account id. Skip.");
                return Ok(BlockVoteCalls::default());
            };
            get_vote_calls_in_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                events,
                is_batch,
                is_multisig,
                is_multisig_executed,
                true,
                is_successful,
                signer,
                &real,
                call,
            )?
        }
        ProxyCall::proxy_announced {
            delegate: _,
            real,
            force_proxy_type: _,
            call,
        } => {
            let Some(real) = get_account_id_from_multi(real)? else {
                log::warn!("Proxy real account is not account id. Skip.");
                return Ok(BlockVoteCalls::default());
            };
            get_vote_calls_in_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                events,
                is_batch,
                is_multisig,
                is_multisig_executed,
                true,
                is_successful,
                signer,
                &real,
                call,
            )?
        }
        _ => BlockVoteCalls::default(),
    };
    Ok(vote_calls)
}

#[allow(clippy::too_many_arguments)]
fn get_vote_calls_in_utility_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    events: &ExtrinsicEvents<PolkadotConfig>,
    _is_batch: bool,
    is_multisig: bool,
    is_multisig_executed: bool,
    is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    voter: &AccountId,
    call: &UtilityCall,
) -> anyhow::Result<BlockVoteCalls> {
    log::trace!("Inspect utility call.");
    let mut vote_calls = BlockVoteCalls::default();
    match call {
        UtilityCall::batch { calls } => {
            for call in calls {
                let mut to_append = get_vote_calls_in_call(
                    network_id,
                    block,
                    extrinsic_index,
                    extrinsic_hash,
                    events,
                    true,
                    is_multisig,
                    is_multisig_executed,
                    is_proxy,
                    is_successful,
                    signer,
                    voter,
                    call,
                )?;
                vote_calls.append(&mut to_append);
            }
        }
        UtilityCall::batch_all { calls } => {
            for call in calls {
                let mut to_append = get_vote_calls_in_call(
                    network_id,
                    block,
                    extrinsic_index,
                    extrinsic_hash,
                    events,
                    true,
                    is_multisig,
                    is_multisig_executed,
                    is_proxy,
                    is_successful,
                    signer,
                    voter,
                    call,
                )?;
                vote_calls.append(&mut to_append);
            }
        }
        UtilityCall::force_batch { calls } => {
            for call in calls {
                let mut to_append = get_vote_calls_in_call(
                    network_id,
                    block,
                    extrinsic_index,
                    extrinsic_hash,
                    events,
                    true,
                    is_multisig,
                    is_multisig_executed,
                    is_proxy,
                    is_successful,
                    signer,
                    voter,
                    call,
                )?;
                vote_calls.append(&mut to_append);
            }
        }
        _ => (),
    };
    Ok(vote_calls)
}

#[allow(clippy::too_many_arguments)]
fn get_vote_calls_in_multisig_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    events: &ExtrinsicEvents<PolkadotConfig>,
    is_batch: bool,
    _is_multisig: bool,
    _is_multisig_executed: bool,
    is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    _voter: &AccountId,
    call: &MultisigCall,
) -> anyhow::Result<BlockVoteCalls> {
    log::trace!("Inspect multisig call.");
    let vote_calls = match call {
        MultisigCall::as_multi_threshold_1 {
            other_signatories,
            call,
        } => {
            let encoded_call = call.encode();
            let call_hash = sp_core::blake2_256(&encoded_call);
            let is_executed = has_multisig_executed_event(events, &call_hash)?;
            let voter = AccountId::multisig_account_id(
                signer,
                &other_signatories
                    .iter()
                    .map(|s| AccountId::from(s.0))
                    .collect::<Vec<_>>(),
                1,
            );
            get_vote_calls_in_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                events,
                is_batch,
                true,
                is_executed,
                is_proxy,
                is_successful,
                signer,
                &voter,
                call,
            )?
        }
        MultisigCall::as_multi {
            threshold,
            other_signatories,
            maybe_timepoint: _,
            call,
            max_weight: _,
        } => {
            let encoded_call = call.encode();
            let call_hash = sp_core::blake2_256(&encoded_call);
            let is_executed = has_multisig_executed_event(events, &call_hash)?;
            let voter = AccountId::multisig_account_id(
                signer,
                &other_signatories
                    .iter()
                    .map(|s| AccountId::from(s.0))
                    .collect::<Vec<_>>(),
                *threshold,
            );
            get_vote_calls_in_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                events,
                is_batch,
                true,
                is_executed,
                is_proxy,
                is_successful,
                signer,
                &voter,
                call,
            )?
        }
        _ => BlockVoteCalls::default(),
    };
    Ok(vote_calls)
}

#[allow(clippy::too_many_arguments)]
fn get_vote_calls_in_call(
    network_id: u32,
    block: &Block,
    extrinsic_index: u32,
    extrinsic_hash: &str,
    events: &ExtrinsicEvents<PolkadotConfig>,
    is_batch: bool,
    is_multisig: bool,
    is_multisig_executed: bool,
    is_proxy: bool,
    is_successful: bool,
    signer: &AccountId,
    voter: &AccountId,
    call: &RuntimeCall,
) -> anyhow::Result<BlockVoteCalls> {
    let vote_calls = match call {
        RuntimeCall::Utility(utility_call) => get_vote_calls_in_utility_call(
            network_id,
            block,
            extrinsic_index,
            extrinsic_hash,
            events,
            is_batch,
            is_multisig,
            is_multisig_executed,
            is_proxy,
            is_successful,
            signer,
            voter,
            utility_call,
        )?,
        RuntimeCall::Proxy(proxy_call) => get_vote_calls_in_proxy_call(
            network_id,
            block,
            extrinsic_index,
            extrinsic_hash,
            events,
            is_batch,
            is_multisig,
            is_multisig_executed,
            is_proxy,
            is_successful,
            signer,
            voter,
            proxy_call,
        )?,
        RuntimeCall::Multisig(multisig_call) => get_vote_calls_in_multisig_call(
            network_id,
            block,
            extrinsic_index,
            extrinsic_hash,
            events,
            is_batch,
            is_multisig,
            is_multisig_executed,
            is_proxy,
            is_successful,
            signer,
            voter,
            multisig_call,
        )?,
        RuntimeCall::ConvictionVoting(conviction_voting_call) => {
            let mut vote_calls = BlockVoteCalls::default();
            if let Some(vote_call) = get_vote_call_in_conviction_voting_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                is_batch,
                is_multisig,
                is_multisig_executed,
                is_proxy,
                is_successful,
                signer,
                voter,
                conviction_voting_call,
            )? {
                vote_calls.vote_calls.push(vote_call);
            }
            if let Some(remove_vote_call) = get_remove_vote_call_in_conviction_voting_call(
                network_id,
                block,
                extrinsic_index,
                extrinsic_hash,
                is_batch,
                is_multisig,
                is_multisig_executed,
                is_proxy,
                is_successful,
                signer,
                voter,
                conviction_voting_call,
            )? {
                vote_calls.remove_vote_calls.push(remove_vote_call);
            }
            vote_calls
        }
        _ => BlockVoteCalls::default(),
    };
    Ok(vote_calls)
}

fn get_extrinsic_signer(address_bytes: Option<&[u8]>) -> anyhow::Result<Option<AccountId>> {
    let Some(address_bytes) = address_bytes else {
        return Ok(None);
    };
    let signer_multi_address = MultiAddress::<AccountId, ()>::decode(&mut &address_bytes[..])?;
    let signer = match signer_multi_address {
        MultiAddress::Id(account_id) => Some(account_id),
        _ => None,
    };
    Ok(signer)
}

fn get_account_id_from_multi(
    multiaddress: &MultiAddress<AccountId32, ()>,
) -> anyhow::Result<Option<AccountId>> {
    let signer = match multiaddress {
        MultiAddress::Id(account_id) => Some(AccountId::from(account_id.0)),
        _ => None,
    };
    Ok(signer)
}

pub(super) async fn get_vote_calls_in_block(
    network_id: u32,
    block: &Block,
    substrate_block: &SubstrateBlock,
) -> anyhow::Result<BlockVoteCalls> {
    let extrinsics = substrate_block.extrinsics().await?;
    let mut block_vote_calls = BlockVoteCalls::default();

    for proxy_extrinsic in extrinsics.find::<ProxyExtrinsic>() {
        let proxy_extrinsic = proxy_extrinsic?;
        let tx_hash = hex::encode(proxy_extrinsic.details.hash());
        let events = proxy_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(proxy_extrinsic.details.address_bytes())? else {
            log::warn!("Proxy extrinsic is not signed. Skip.");
            continue;
        };
        let Some(voter) = get_account_id_from_multi(&proxy_extrinsic.value.real)? else {
            log::warn!("Cannot extract the real account for proxy.");
            continue;
        };
        log::trace!("Proxy extrinsic is successful: {is_successful}");
        let mut extrinsic_vote_calls = get_vote_calls_in_call(
            network_id,
            block,
            proxy_extrinsic.details.index(),
            tx_hash.as_str(),
            &events,
            false,
            false,
            false,
            true,
            is_successful,
            &signer,
            &voter,
            &(*proxy_extrinsic.value.call as RuntimeCall),
        )?;
        block_vote_calls.append(&mut extrinsic_vote_calls);
    }

    for proxy_announced_extrinsic in extrinsics.find::<ProxyAnnouncedExtrinsic>() {
        let proxy_announced_extrinsic = proxy_announced_extrinsic?;
        let tx_hash = hex::encode(proxy_announced_extrinsic.details.hash());
        let events = proxy_announced_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(proxy_announced_extrinsic.details.address_bytes())?
        else {
            log::warn!("ProxyAnnounced extrinsic is not signed. Skip.");
            continue;
        };
        let Some(voter) = get_account_id_from_multi(&proxy_announced_extrinsic.value.real)? else {
            log::warn!("Cannot extract the real account for proxy.");
            continue;
        };
        log::trace!("ProxyAnnounced extrinsic is successful: {is_successful}");
        let mut extrinsic_vote_calls = get_vote_calls_in_call(
            network_id,
            block,
            proxy_announced_extrinsic.details.index(),
            tx_hash.as_str(),
            &events,
            false,
            false,
            false,
            true,
            is_successful,
            &signer,
            &voter,
            &(*proxy_announced_extrinsic.value.call as RuntimeCall),
        )?;
        block_vote_calls.append(&mut extrinsic_vote_calls);
    }

    for force_batch_extrinsic in extrinsics.find::<ForceBatchExtrinsic>() {
        let force_batch_extrinsic = force_batch_extrinsic?;
        let tx_hash = hex::encode(force_batch_extrinsic.details.hash());
        let events = force_batch_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(force_batch_extrinsic.details.address_bytes())?
        else {
            log::warn!("ForceBatch extrinsic is not signed. Skip.");
            continue;
        };
        log::trace!("ForceBatch extrinsic is successful: {is_successful}");
        for call in force_batch_extrinsic.value.calls.iter() {
            let mut extrinsic_vote_calls = get_vote_calls_in_call(
                network_id,
                block,
                force_batch_extrinsic.details.index(),
                tx_hash.as_str(),
                &events,
                true,
                false,
                false,
                false,
                is_successful,
                &signer,
                &signer,
                call,
            )?;
            block_vote_calls.append(&mut extrinsic_vote_calls);
        }
    }
    for batch_all_extrinsic in extrinsics.find::<BatchAllExtrinsic>() {
        let batch_all_extrinsic = batch_all_extrinsic?;
        let tx_hash = hex::encode(batch_all_extrinsic.details.hash());
        let events = batch_all_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(batch_all_extrinsic.details.address_bytes())?
        else {
            log::warn!("BatchAll extrinsic is not signed. Skip.");
            continue;
        };
        log::trace!("BatchAll extrinsic is successful: {is_successful}");
        for call in batch_all_extrinsic.value.calls.iter() {
            let mut extrinsic_vote_calls = get_vote_calls_in_call(
                network_id,
                block,
                batch_all_extrinsic.details.index(),
                tx_hash.as_str(),
                &events,
                true,
                false,
                false,
                false,
                is_successful,
                &signer,
                &signer,
                call,
            )?;
            block_vote_calls.append(&mut extrinsic_vote_calls);
        }
    }
    for batch_extrinsic in extrinsics.find::<BatchExtrinsic>() {
        let batch_extrinsic = batch_extrinsic?;
        let tx_hash = hex::encode(batch_extrinsic.details.hash());
        let events = batch_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(batch_extrinsic.details.address_bytes())? else {
            log::warn!("Batch extrinsic is not signed. Skip.");
            continue;
        };
        log::trace!("Batch extrinsic is successful: {is_successful}");
        for call in batch_extrinsic.value.calls.iter() {
            let mut extrinsic_vote_calls = get_vote_calls_in_call(
                network_id,
                block,
                batch_extrinsic.details.index(),
                tx_hash.as_str(),
                &events,
                true,
                false,
                false,
                false,
                is_successful,
                &signer,
                &signer,
                call,
            )?;
            block_vote_calls.append(&mut extrinsic_vote_calls);
        }
    }

    for as_multi_extrinsic in extrinsics.find::<AsMultiExtrinsic>() {
        let as_multi_extrinsic = as_multi_extrinsic?;
        let tx_hash = hex::encode(as_multi_extrinsic.details.hash());
        let events = as_multi_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(as_multi_extrinsic.details.address_bytes())? else {
            log::warn!("AsMulti extrinsic is not signed. Skip.");
            continue;
        };
        let encoded_call = as_multi_extrinsic.value.call.encode();
        let call_hash = sp_core::blake2_256(&encoded_call);
        let is_executed = has_multisig_executed_event(&events, &call_hash)?;

        log::info!("AsMulti extrinsic executed the multisig. Process.");
        let voter = AccountId::multisig_account_id(
            &signer,
            &as_multi_extrinsic
                .value
                .other_signatories
                .iter()
                .map(|s| AccountId::from(s.0))
                .collect::<Vec<_>>(),
            as_multi_extrinsic.value.threshold,
        );
        log::trace!("AsMulti extrinsic is successful: {is_successful}, is executed: {is_executed}");
        let mut extrinsic_vote_calls = get_vote_calls_in_call(
            network_id,
            block,
            as_multi_extrinsic.details.index(),
            tx_hash.as_str(),
            &events,
            false,
            true,
            is_executed,
            false,
            is_successful,
            &signer,
            &voter,
            &(*as_multi_extrinsic.value.call as RuntimeCall),
        )?;
        block_vote_calls.append(&mut extrinsic_vote_calls);
    }
    for as_multi_threshold_1_extrinsic in extrinsics.find::<AsMultiThreshold1Extrinsic>() {
        let as_multi_threshold_1_extrinsic = as_multi_threshold_1_extrinsic?;
        let tx_hash = hex::encode(as_multi_threshold_1_extrinsic.details.hash());
        let events = as_multi_threshold_1_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) =
            get_extrinsic_signer(as_multi_threshold_1_extrinsic.details.address_bytes())?
        else {
            log::warn!("AsMultiThreshold1 extrinsic is not signed. Skip.");
            continue;
        };
        let encoded_call = as_multi_threshold_1_extrinsic.value.call.encode();
        let call_hash = sp_core::blake2_256(&encoded_call);
        let is_executed = has_multisig_executed_event(&events, &call_hash)?;
        let voter = AccountId::multisig_account_id(
            &signer,
            &as_multi_threshold_1_extrinsic
                .value
                .other_signatories
                .iter()
                .map(|s| AccountId::from(s.0))
                .collect::<Vec<_>>(),
            1,
        );
        log::trace!("AsMulti extrinsic is successful: {is_successful}, is executed: {is_executed}");
        let mut extrinsic_vote_calls = get_vote_calls_in_call(
            network_id,
            block,
            as_multi_threshold_1_extrinsic.details.index(),
            tx_hash.as_str(),
            &events,
            true,
            false,
            is_executed,
            false,
            is_successful,
            &signer,
            &voter,
            &(*as_multi_threshold_1_extrinsic.value.call as RuntimeCall),
        )?;
        block_vote_calls.append(&mut extrinsic_vote_calls);
    }

    for remove_vote_extrinsic in extrinsics.find::<RemoveVoteExtrinsic>() {
        let remove_vote_extrinsic = remove_vote_extrinsic?;
        let tx_hash = hex::encode(remove_vote_extrinsic.details.hash());
        let events = remove_vote_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(remove_vote_extrinsic.details.address_bytes())?
        else {
            log::warn!("RemoveVote extrinsic is not signed. Skip.");
            continue;
        };
        log::trace!("RemoveVote extrinsic is successful: {is_successful}");
        block_vote_calls.remove_vote_calls.push(RemoveVoteCall {
            network_id,
            block: block.clone(),
            extrinsic_index: remove_vote_extrinsic.details.index(),
            extrinsic_hash: tx_hash,
            is_batch: false,
            is_multisig: false,
            is_multisig_executed: false,
            is_proxy: false,
            is_successful,
            signer,
            voter: signer,
            referendum_index: remove_vote_extrinsic.value.index,
        });
    }
    for vote_extrinsic in extrinsics.find::<VoteExtrinsic>() {
        let vote_extrinsic = vote_extrinsic?;
        let tx_hash = hex::encode(vote_extrinsic.details.hash());
        let events = vote_extrinsic.details.events().await?;
        let is_successful = has_extrinsic_success_event(&events)?;
        let Some(signer) = get_extrinsic_signer(vote_extrinsic.details.address_bytes())? else {
            log::warn!("Vote extrinsic is not signed. Skip.");
            continue;
        };
        log::trace!("Vote extrinsic found is successful: {is_successful}");
        block_vote_calls.vote_calls.push(VoteCall {
            network_id,
            block: block.clone(),
            extrinsic_index: vote_extrinsic.details.index(),
            extrinsic_hash: tx_hash,
            is_batch: false,
            is_multisig: false,
            is_multisig_executed: false,
            is_proxy: false,
            is_successful,
            signer,
            voter: signer,
            referendum_index: vote_extrinsic.value.poll_index,
            vote: vote_extrinsic.value.vote.into(),
        });
    }
    Ok(block_vote_calls)
}

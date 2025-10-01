use crate::substrate::account_id::AccountId;
use crate::substrate::vote::Tally;
use frame_support::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ReferendumEvent {
    Submitted {
        referendum_index: u32,
        track_id: u16,
    },
    DecisionDepositPlaced {
        referendum_index: u32,
        amount: u128,
        who: AccountId,
    },
    DecisionDepositRefunded {
        referendum_index: u32,
        amount: u128,
        who: AccountId,
    },
    DepositSlashed {
        amount: u128,
        who: AccountId,
    },
    DecisionStarted {
        referendum_index: u32,
        track_id: u16,
        tally: Tally,
    },
    ConfirmStarted {
        referendum_index: u32,
    },
    ConfirmAborted {
        referendum_index: u32,
    },
    Confirmed {
        referendum_index: u32,
        tally: Tally,
    },
    Approved {
        referendum_index: u32,
    },
    Rejected {
        referendum_index: u32,
        tally: Tally,
    },
    Cancelled {
        referendum_index: u32,
        tally: Tally,
    },
    TimedOut {
        referendum_index: u32,
        tally: Tally,
    },
    Killed {
        referendum_index: u32,
        tally: Tally,
    },
    SubmissionDepositRefunded {
        referendum_index: u32,
        amount: u128,
        who: AccountId,
    },
}

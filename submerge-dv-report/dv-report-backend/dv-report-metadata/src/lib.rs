#[cfg(all(feature = "polkadot", feature = "kusama"))]
compile_error!("You must enable only one of the features: polkadot or kusama.");

#[cfg(feature = "kusama")]
pub use runtime::kusama as metadata;
#[cfg(feature = "kusama")]
pub use runtime::kusama::api::runtime_types::pallet_conviction_voting::vote::AccountVote;
#[cfg(feature = "kusama")]
pub use runtime::kusama::api::runtime_types::staging_kusama_runtime::RuntimeCall;
#[cfg(feature = "kusama")]
pub use runtime::kusama_current as metadata_current;
#[cfg(feature = "polkadot")]
pub use runtime::polkadot as metadata;
#[cfg(feature = "polkadot")]
pub use runtime::polkadot::api::runtime_types::pallet_conviction_voting::vote::AccountVote;
#[cfg(feature = "polkadot")]
pub use runtime::polkadot::api::runtime_types::polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use runtime::polkadot_current as metadata_current;
use subxt::ext::jsonrpsee::core::Serialize;

use dv_report_types::governance::vote::AccountVote as NativeAccountVote;
use dv_report_types::governance::vote::Vote as NativeVote;

mod runtime;

impl<T: Copy + Serialize> From<AccountVote<T>> for NativeAccountVote<T> {
    fn from(account_vote: AccountVote<T>) -> Self {
        (&account_vote).into()
    }
}

impl<T: Copy + Serialize> From<&AccountVote<T>> for NativeAccountVote<T> {
    fn from(account_vote: &AccountVote<T>) -> Self {
        match account_vote {
            AccountVote::Standard { vote, balance } => NativeAccountVote::Standard {
                vote: NativeVote(vote.0),
                balance: *balance,
            },
            AccountVote::Split { aye, nay } => NativeAccountVote::Split {
                aye: *aye,
                nay: *nay,
            },
            AccountVote::SplitAbstain { aye, nay, abstain } => NativeAccountVote::SplitAbstain {
                aye: *aye,
                nay: *nay,
                abstain: *abstain,
            },
        }
    }
}

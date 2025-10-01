use dv_report_metadata::metadata::api::referenda::events::{
    Approved, Cancelled, ConfirmAborted, ConfirmStarted, Confirmed, DecisionDepositPlaced,
    DecisionDepositRefunded, DecisionStarted, DepositSlashed, Killed, Rejected,
    SubmissionDepositRefunded, Submitted, TimedOut,
};
use dv_report_types::substrate::account_id::AccountId;
use dv_report_types::substrate::event::ReferendumEvent;
use dv_report_types::substrate::vote::Tally;

pub(super) async fn get_referendum_events_in_block(
    block: &crate::vote::SubstrateBlock,
) -> anyhow::Result<Vec<ReferendumEvent>> {
    let mut referendum_events = Vec::new();
    let block_events = block.events().await?;
    for event in block_events.find::<Submitted>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Submitted {
            referendum_index: event.index,
            track_id: event.track,
        });
    }
    for event in block_events.find::<DecisionDepositPlaced>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::DecisionDepositPlaced {
            referendum_index: event.index,
            amount: event.amount,
            who: AccountId::from(event.who.0),
        });
    }
    for event in block_events.find::<DecisionDepositRefunded>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::DecisionDepositRefunded {
            referendum_index: event.index,
            amount: event.amount,
            who: AccountId::from(event.who.0),
        });
    }
    for event in block_events.find::<DepositSlashed>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::DepositSlashed {
            amount: event.amount,
            who: AccountId::from(event.who.0),
        });
    }
    for event in block_events.find::<DecisionStarted>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::DecisionStarted {
            referendum_index: event.index,
            track_id: event.track,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<ConfirmStarted>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::ConfirmStarted {
            referendum_index: event.index,
        });
    }
    for event in block_events.find::<ConfirmAborted>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::ConfirmAborted {
            referendum_index: event.index,
        });
    }
    for event in block_events.find::<Confirmed>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Confirmed {
            referendum_index: event.index,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<Approved>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Approved {
            referendum_index: event.index,
        });
    }
    for event in block_events.find::<Rejected>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Rejected {
            referendum_index: event.index,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<Cancelled>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Cancelled {
            referendum_index: event.index,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<TimedOut>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::TimedOut {
            referendum_index: event.index,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<Killed>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::Killed {
            referendum_index: event.index,
            tally: Tally {
                ayes: event.tally.ayes,
                nays: event.tally.nays,
                support: event.tally.support,
            },
        });
    }
    for event in block_events.find::<SubmissionDepositRefunded>() {
        let event = event?;
        referendum_events.push(ReferendumEvent::SubmissionDepositRefunded {
            referendum_index: event.index,
            amount: event.amount,
            who: AccountId::from(event.who.0),
        });
    }
    Ok(referendum_events)
}

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Vote(pub u8);

#[derive(Clone, Debug, Serialize)]
pub enum AccountVote<T: Serialize> {
    Standard { vote: Vote, balance: T },
    Split { aye: T, nay: T },
    SplitAbstain { aye: T, nay: T, abstain: T },
}

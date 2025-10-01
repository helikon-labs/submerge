use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Sequence, Serialize, Deserialize, EnumIter)]
pub enum Track {
    Root,
    WhitelistedCaller,
    WishForChange,
    // general admin
    StakingAdmin,
    Treasurer,
    LeaseAdmin,
    FellowshipAdmin,
    GeneralAdmin,
    AuctionAdmin,
    // referendum admins
    ReferendumCanceller,
    ReferendumKiller,
    // limited treasury spenders
    SmallTipper,
    BigTipper,
    SmallSpender,
    MediumSpender,
    BigSpender,
}

impl Track {
    pub fn id(&self) -> u16 {
        match self {
            Track::Root => 0,
            Track::WhitelistedCaller => 1,
            Track::WishForChange => 2,
            // general admin
            Track::StakingAdmin => 10,
            Track::Treasurer => 11,
            Track::LeaseAdmin => 12,
            Track::FellowshipAdmin => 13,
            Track::GeneralAdmin => 14,
            Track::AuctionAdmin => 15,
            // referendum admins
            Track::ReferendumCanceller => 20,
            Track::ReferendumKiller => 21,
            // limited treasury spenders
            Track::SmallTipper => 30,
            Track::BigTipper => 31,
            Track::SmallSpender => 32,
            Track::MediumSpender => 33,
            Track::BigSpender => 34,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Track::Root => "Root",
            Track::WhitelistedCaller => "Whitelisted Caller",
            Track::WishForChange => "Wish For Change",
            // general admin
            Track::StakingAdmin => "Staking Admin",
            Track::Treasurer => "Treasurer",
            Track::LeaseAdmin => "Lease Admin",
            Track::FellowshipAdmin => "Fellowship Admin",
            Track::GeneralAdmin => "General Admin",
            Track::AuctionAdmin => "Auction Admin",
            // referendum admins
            Track::ReferendumCanceller => "Referendum Canceller",
            Track::ReferendumKiller => "Referendum Killer",
            // limited treasury spenders
            Track::SmallTipper => "Small Tipper",
            Track::BigTipper => "Big Tipper",
            Track::SmallSpender => "Small Spender",
            Track::MediumSpender => "Medium Spender",
            Track::BigSpender => "Big Spender",
        }
    }

    pub fn from_id(id: u16) -> Track {
        match id {
            0 => Track::Root,
            1 => Track::WhitelistedCaller,
            2 => Track::WishForChange,
            10 => Track::StakingAdmin,
            11 => Track::Treasurer,
            12 => Track::LeaseAdmin,
            13 => Track::FellowshipAdmin,
            14 => Track::GeneralAdmin,
            15 => Track::AuctionAdmin,
            20 => Track::ReferendumCanceller,
            21 => Track::ReferendumKiller,
            30 => Track::SmallTipper,
            31 => Track::BigTipper,
            32 => Track::SmallSpender,
            33 => Track::MediumSpender,
            34 => Track::BigSpender,
            _ => panic!("Unknown track id: {id}"),
        }
    }

    pub fn is_dv_track(&self) -> bool {
        match self {
            Track::Root => false,
            Track::WhitelistedCaller => false,
            Track::WishForChange => true,
            // general admin
            Track::StakingAdmin => false,
            Track::Treasurer => true,
            Track::LeaseAdmin => false,
            Track::FellowshipAdmin => false,
            Track::GeneralAdmin => false,
            Track::AuctionAdmin => false,
            // referendum admins
            Track::ReferendumCanceller => false,
            Track::ReferendumKiller => false,
            // limited treasury spenders
            Track::SmallTipper => true,
            Track::BigTipper => true,
            Track::SmallSpender => true,
            Track::MediumSpender => true,
            Track::BigSpender => true,
        }
    }

    pub fn get_dv_tracks() -> Vec<Track> {
        let mut maybe_track = Track::first();
        let mut dv_tracks = Vec::new();
        while let Some(track) = maybe_track {
            if track.is_dv_track() {
                dv_tracks.push(track);
            }
            maybe_track = track.next();
        }
        dv_tracks
    }

    pub fn all() -> Vec<Track> {
        Self::iter().collect()
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackRow {
    pub network_id: i32,
    pub id: i32,
    pub name: String,
}

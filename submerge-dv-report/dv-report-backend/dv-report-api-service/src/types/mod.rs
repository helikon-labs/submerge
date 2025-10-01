use dv_report_types::governance::referendum::ReferendumStatusRow;
use dv_report_types::substrate::block::Block;
use dv_report_types::substrate::track::TrackRow;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReferendumDTO {
    pub network_id: u32,
    pub index: u32,
    pub track: TrackRow,
    pub submission_block: Block,
    pub status: ReferendumStatusRow,
    pub is_retracted: bool,
}

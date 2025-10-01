use chrono::{DateTime, Utc};
use frame_support::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PolkassemblyReferendumComment {
    pub id: String,
    pub content: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub proposer: Option<String>,
    pub username: String,
    pub replies: Vec<PolkassemblyReferendumComment>,
    pub comment_source: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PolkassemblyReferendumCommentListResponse {
    pub index: u32,
    pub comments: Vec<PolkassemblyReferendumComment>,
    pub proposer: String,
    pub comments_count: u32,
}

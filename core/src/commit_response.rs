use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CommitResponse {
    pub ok: bool,
    pub preview_id: String,
    pub committed_at: DateTime<Utc>,
    pub rows_affected: u64,
}

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::cache::Cache;

pub async fn stats(State(cache): State<Cache>) -> impl IntoResponse {
    cache.run_pending_tasks().await;
    let bytes = Bytes(cache.weighted_size());
    tracing::info!(bytes = bytes.0, "BYTES");
    (
        StatusCode::OK,
        Json(Stats {
            //TODO: read max size from config but I wanted to only read
            // cache as an excuse to implement `FromRef` and
            // use the State extractor.
            size: format!("{:.2} MiB/1024 MiB", bytes.as_mib()),
            entries: cache.entry_count(),
        }),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Bytes(u64);
impl Bytes {
    const fn as_mib(self) -> u64 {
        self.0 / (1024 * 1024)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Stats {
    size: String,
    entries: u64,
}

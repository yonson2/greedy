mod processor;
mod stats;

use crate::{
    cache::{self, Cache},
    config::Config,
};
use axum::{extract::FromRef, http::StatusCode, routing::get, Json, Router};
use processor::{preload, process_and_serve};
use serde::{Deserialize, Serialize};
use stats::stats;
use tower_http::trace::TraceLayer;

pub type Result<T, E = crate::error::Error> = std::result::Result<T, E>;

/// `serve` setps up dependencies and starts up the http server
/// ready to serve requests.
///
/// # Errors
/// # Panics
///  - When setting up an invalid `tokio` `TcpListener`
///  - When `axum` can't serve our app.
pub async fn serve(config: Config) -> Result<()> {
    let whitelist = config.whitelist;
    let state = ApiState {
        whitelist,
        cache: cache::new(&config.cache),
    };

    let app = routes(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", &config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn routes(state: ApiState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/*url", get(process_and_serve))
        .route("/preload/*url", get(preload))
        .route("/stats", get(stats))
        .with_state(state)
}

async fn index() -> (StatusCode, Json<ApiMessage>) {
    (
        StatusCode::IM_A_TEAPOT,
        Json(ApiMessage {
            message: "Hello".to_string(),
        }),
    )
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMessage {
    message: String,
}

#[derive(Debug, Clone)]
pub struct ApiState {
    pub whitelist: Vec<String>,
    pub cache: Cache,
}

impl FromRef<ApiState> for Cache {
    fn from_ref(state: &ApiState) -> Self {
        state.cache.clone()
    }
}

use crate::config::Config;
use axum::{http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

pub type Result<T, E = crate::error::Error> = std::result::Result<T, E>;

/// `serve` setps up dependencies and starts up the http server
/// ready to serve requests.
///
/// # Errors
/// # Panics
///  - When setting up a `tokio` `TcpListener`
///  - When serving our app through `axum`
pub async fn serve(config: Config) -> Result<()> {
    let app = routes().layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn routes() -> Router {
    Router::new().route("/", get(index))
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
struct ApiMessage {
    message: String,
}

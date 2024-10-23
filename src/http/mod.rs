use crate::{cache, config::Config};
use axum::{http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
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
    let app = routes(&config).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", &config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn routes(config: &Config) -> Router {
    Router::new()
        .route("/", get(index))
        .with_state(cache::new(&config.cache))
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

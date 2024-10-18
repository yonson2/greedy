use greedy::{config, error::Error, http};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let c = config::Config::new().expect("Valid configuration");
    http::serve(c).await.unwrap();

    Ok(())
}

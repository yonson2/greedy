use std::error::Error;

mod config;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello, world!");

    let c = config::Config::new()?;
    tracing::info!("{:#?}", &c.whitelist);

    Ok(())
}

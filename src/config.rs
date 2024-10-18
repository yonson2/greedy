use std::env;

use config::{Config as C, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub whitelist: Vec<String>,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUST_ENV").unwrap_or_else(|_| "development".into());
        let c = C::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()?;
        c.try_deserialize()
    }
}

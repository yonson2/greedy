//! Contains the means to obtain configuration info used at runtime.
use std::env;

use config::{Config as C, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Cache {
    pub capacity: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub whitelist: Vec<String>,
    pub cache: Cache,
}

impl Config {
    /// Returns a `Config` struct with configuration info.
    ///
    /// # Errors
    /// When build errors are present (for example, file not found)
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUST_ENV").unwrap_or_else(|_| "development".into());
        let c = C::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{run_mode}")).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                config::Environment::with_prefix("GREEDY")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()?;
        c.try_deserialize()
    }
}

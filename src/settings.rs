use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "Development".into());

        let s = Config::builder()
            // Add in the default configuration file
            .add_source(File::with_name("./config/Default"))
            // Add in the current environment configuration file
            // Defaults to 'development' env
            .add_source(
                File::with_name(&format!("./config/{}", run_mode))
                    .required(false),
            )
            .add_source(Environment::with_prefix("SKID"))
            .build()?;

        s.try_deserialize()
    }
}

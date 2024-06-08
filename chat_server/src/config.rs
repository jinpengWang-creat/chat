use anyhow::{anyhow, Result};
use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

// default port is 8080
// default ip is 0.0.0.0
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_ip")]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl TryFrom<Config> for AppConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        // Convert the Config struct to AppConfig here
        let server = config.get::<ServerConfig>("server")?;
        Ok(AppConfig { server })
    }
}

impl AppConfig {
    // load config from file, the sequence is :
    // 1. load from default
    // 2. load from config.yml
    // 3. load from etc/app/config.yml
    // 4. load from env
    // 4 is the highest priority and 1 is the lowest priority
    pub fn load() -> Result<Self> {
        let mut builder = ConfigBuilder::<DefaultState>::default();
        if File::open("config.yml").is_ok() {
            builder = builder.add_source(config::File::with_name("config"));
        }
        if File::open("etc/app/config.yml").is_ok() {
            builder = builder.add_source(config::File::with_name("etc/app/config"));
        }
        builder = builder.add_source(config::Environment::default());
        let config = builder.build().unwrap();
        config
            .try_into()
            .map_err(|e| anyhow!(format!("parse config error: {:?}", e)))
    }
}

fn default_port() -> u16 {
    8080
}

fn default_ip() -> String {
    "0.0.0.0".to_string()
}

#[cfg(test)]
mod config_test {
    use super::*;

    #[test]
    fn test_config() {
        let config = AppConfig::load().unwrap();
        println!("{:?}", config)
    }
}

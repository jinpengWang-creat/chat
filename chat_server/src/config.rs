use anyhow::{anyhow, Result};
use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,

    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default = "default_sk")]
    pub sk: String,
    #[serde(default = "default_pk")]
    pub pk: String,
}

// default port is 8080
// default ip is 0.0.0.0
#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default = "default_ip")]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_db_url")]
    pub db_url: String,
}

impl TryFrom<Config> for AppConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        // Convert the Config struct to AppConfig here
        let server = config.get::<ServerConfig>("server")?;
        let auth = config.get::<AuthConfig>("auth")?;
        Ok(AppConfig { server, auth })
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
    6666
}

fn default_ip() -> String {
    "0.0.0.0".to_string()
}

fn default_db_url() -> String {
    "postgres://postgres:postgres@localhost:5432/chat".to_string()
}

fn default_pk() -> String {
    "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAfM+lwNHj6TRJ3EGP38lIJcOo9Dlt2u2JzcwWMbu7jQY=
-----END PUBLIC KEY-----"
        .to_string()
}

fn default_sk() -> String {
    "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIDnxJGEJGoW+mNKHn4vRY1V6BQ3MglSQSuZ8featmyC4
-----END PRIVATE KEY-----"
        .to_string()
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

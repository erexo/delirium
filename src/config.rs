use std::net::{IpAddr, Ipv4Addr};

use anyhow::{Context, Result};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub api: Api,
    pub jwt: Jwt,
    pub database: Database,
    pub debug: Debug,
}

#[derive(Deserialize, Serialize)]
pub struct Api {
    pub address: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Jwt {
    pub secret: String,
    pub subject: Option<String>,
    pub audience: Option<String>,
    pub time: usize,
    pub refresh_time: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Database {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub connections: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Debug {
    pub log: LevelFilter,
    pub swagger: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: Default::default(),
            jwt: Default::default(),
            database: Default::default(),
            debug: Default::default(),
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            address: Ipv4Addr::new(127, 0, 0, 1).into(),
            port: 80,
        }
    }
}

impl Default for Jwt {
    fn default() -> Self {
        Self {
            secret: Default::default(),
            subject: None,
            audience: None,
            time: 15 * 60,
            refresh_time: 7 * 24 * 60 * 60,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            user: Default::default(),
            password: Default::default(),
            database: "delirium".to_owned(),
            connections: 10,
        }
    }
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            log: LevelFilter::Warn,
            swagger: false,
        }
    }
}

pub fn new() -> Result<Config> {
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("DELIRIUM_").split("_"))
        .extract()
        .context("config")
}

impl Config {
    pub fn rocket(&self) -> rocket::config::Config {
        let mut ret = rocket::config::Config::default();
        ret.address = self.api.address;
        ret.port = self.api.port;
        ret
    }
}

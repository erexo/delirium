use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::metadata::LevelFilter;

lazy_static! {
    static ref CONFIG: Config = new();
}

pub fn get() -> &'static Config {
    &CONFIG
}

// used by validation macro
pub fn field_length() -> (usize, usize) {
    let cfg = &get().validation;
    (cfg.min_length, cfg.max_length)
}

fn new() -> Config {
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("DELIRIUM_").split("_"))
        .extract()
        .expect("config")
}

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub api: Api,
    pub jwt: Jwt,
    pub database: Database,
    #[serde(deserialize_with = "deserialize_str_map")]
    pub worlds: HashMap<u32, String>,
    pub account: Account,
    pub character: Character,
    pub validation: Validation,
    pub debug: Debug,
}

#[derive(Deserialize, Serialize)]
pub struct Api {
    pub name: String,
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
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub max_characters: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub insta_delete_below: i32,
    pub new: NewCharacter,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCharacter {
    pub health: u32,
    pub mana: u32,
    pub soul: u32,
    pub cap: u32,
    pub town: u32,
    pub pos_x: u32,
    pub pos_y: u32,
    pub pos_z: u32,
    #[serde(deserialize_with = "deserialize_str_map")]
    pub vocations: HashMap<u32, NewVocation>,
}

#[derive(Deserialize, Serialize)]
pub struct NewVocation {
    pub vocation: u32,
    pub looktype: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validation {
    pub min_length: usize,
    pub max_length: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Debug {
    pub log: Log,
    pub swagger: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Log {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Log> for LevelFilter {
    fn from(value: Log) -> Self {
        match value {
            Log::Error => LevelFilter::ERROR,
            Log::Warn => LevelFilter::WARN,
            Log::Info => LevelFilter::INFO,
            Log::Debug => LevelFilter::DEBUG,
            Log::Trace => LevelFilter::TRACE,
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self {
            name: "Delirium".to_owned(),
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

impl Default for Account {
    fn default() -> Self {
        Self {
            max_characters: 10,
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            insta_delete_below: 10,
            new: Default::default(),
        }
    }
}

impl Default for NewCharacter {
    fn default() -> Self {
        Self {
            health: 200,
            mana: 0,
            soul: 100,
            cap: 420,
            town: 1,
            pos_x: 1000,
            pos_y: 1000,
            pos_z: 7,
            vocations: HashMap::new(),
        }
    }
}

impl Default for NewVocation {
    fn default() -> Self {
        Self {
            vocation: 0,
            looktype: 100,
        }
    }
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            min_length: 4,
            max_length: 32,
        }
    }
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            log: Log::Warn,
            swagger: false,
        }
    }
}

fn deserialize_str_map<'de, D, V>(deserializer: D) -> Result<HashMap<u32, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let s: HashMap<String, V> = Deserialize::deserialize(deserializer)?;
    let mut ret = HashMap::new();
    for (k, v) in s {
        let k = k
            .parse()
            .map_err(|_| serde::de::Error::custom("key is not u32"))?;
        ret.insert(k, v);
    }
    Ok(ret)
}

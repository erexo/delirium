use std::{collections::HashMap, sync::Mutex};

use anyhow::{Context, Result};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::{debug, warn};
use rand::random;
use serde::{Deserialize, Serialize};

use crate::{config, utils::time};

pub struct Service {
    refresh_tokens: Mutex<HashMap<u128, i64>>,
}

pub fn new() -> Service {
    Service {
        refresh_tokens: Mutex::new(HashMap::new()),
    }
}

impl Service {
    pub fn register(&self, cfg: &config::Jwt, aid: i64) -> Result<(String, String)> {
        let refresh_id = self.refresh_id(aid);
        let now = time::now();
        let token = self.internal_refresh(cfg, refresh_id, now)?;
        let refresh_token = encode(
            &Header::default(),
            &RefreshClaims {
                aud: cfg.audience.clone().unwrap_or_default(),
                sub: cfg.subject.clone().unwrap_or_default(),
                iat: now,
                exp: now + cfg.refresh_time,
                rid: refresh_id,
            },
            &EncodingKey::from_secret(cfg.secret.as_bytes()),
        )
        .context("refresh token")?;

        debug!("Generated token pair for '{aid}'");
        Ok((token, refresh_token))
    }

    pub fn unregister_token(&self, rid: u128) {
        let mut rtokens = self.refresh_tokens.lock().expect("lock");
        if let Some(aid) = rtokens.remove(&rid) {
            debug!("Unregistered '{aid}'");
        } else {
            warn!("Unable to unregister token '{rid}'");
        }
    }

    pub fn refresh(&self, cfg: &config::Jwt, rid: u128) -> Result<String> {
        self.internal_refresh(cfg, rid, time::now())
    }

    fn internal_refresh(&self, cfg: &config::Jwt, rid: u128, now: usize) -> Result<String> {
        let aid = *self
            .refresh_tokens
            .lock()
            .expect("lock")
            .get(&rid)
            .context("No refresh token found")?;
        encode(
            &Header::default(),
            &Claims {
                aud: cfg.audience.clone().unwrap_or_default(),
                sub: cfg.subject.clone().unwrap_or_default(),
                iat: now,
                exp: now + cfg.time,
                aid,
            },
            &EncodingKey::from_secret(cfg.secret.as_bytes()),
        )
        .context("token")
    }

    fn refresh_id(&self, aid: i64) -> u128 {
        let mut rtokens = self.refresh_tokens.lock().expect("lock");
        let mut refresh_id = random::<u128>();
        while rtokens.contains_key(&refresh_id) {
            refresh_id = random::<u128>();
        }
        if let Some((&k, _)) = rtokens.iter().find(|(_, v)| *v == &aid) {
            rtokens.remove(&k);
            debug!("Removed access token for '{aid}'");
        }
        rtokens.insert(refresh_id, aid);
        refresh_id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    aud: String,
    sub: String,
    iat: usize,
    exp: usize,
    aid: i64,
}

impl Claims {
    pub fn aid(&self) -> i64 {
        self.aid
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    aud: String,
    sub: String,
    iat: usize,
    exp: usize,
    rid: u128,
}

impl RefreshClaims {
    pub fn rid(&self) -> u128 {
        self.rid
    }
}

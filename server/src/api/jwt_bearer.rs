use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use poem::Request;
use poem_openapi::{auth::Bearer, SecurityScheme};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    config,
    services::jwt::{AccountClaims, RefreshClaims},
};

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    rename = "account_token",
    checker = "account_api_checker"
)]
pub struct JwtAccountId(pub i32);

impl From<JwtAccountId> for i32 {
    fn from(value: JwtAccountId) -> Self {
        value.0
    }
}

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    rename = "refresh_token",
    checker = "refresh_api_checker"
)]
pub struct JwtRefreshId(pub JwtRefreshIdData);

pub struct JwtRefreshIdData {
    pub rid: u128,
    pub refresh_token: String,
}

async fn account_api_checker(_: &Request, bearer: Bearer) -> Option<i32> {
    match validate::<AccountClaims>(&bearer.token) {
        Ok(claims) => Some(claims.aid()),
        Err(err) => {
            debug!("Jwt failed: {}", err);
            None
        }
    }
}

async fn refresh_api_checker(_: &Request, bearer: Bearer) -> Option<JwtRefreshIdData> {
    match validate::<RefreshClaims>(&bearer.token) {
        Ok(claims) => Some(JwtRefreshIdData {
            rid: claims.rid(),
            refresh_token: bearer.token.clone(),
        }),
        Err(err) => {
            debug!("JwtRefresh failed: {}", err);
            None
        }
    }
}

fn validate<T: DeserializeOwned>(token: &str) -> core::result::Result<T, String> {
    let cfg = &config::get().jwt;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.sub = cfg.subject.clone();
    if let Some(ref audience) = cfg.audience {
        validation.set_audience(&[audience.clone()]);
    }
    match decode::<T>(
        token,
        &DecodingKey::from_secret(cfg.secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => Ok(data.claims),
        Err(err) => Err(err.to_string()),
    }
}

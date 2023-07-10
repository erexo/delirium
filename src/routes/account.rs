use anyhow::Context;
use regex::Regex;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket::{Route, State};
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, MySql, Pool};

use crate::config::{self, Config};
use crate::services::jwt;
use crate::utils::time;

use super::{Error, Result};
use super::{JwtAccountId, JwtRefreshId};

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![create, login, refresh_token, logout, account]
}

#[openapi]
#[put("/create", data = "<payload>")]
async fn create(
    payload: Json<Create>,
    config: &State<Config>,
    db: &State<Pool<MySql>>,
    jwt: &State<jwt::Service>,
) -> Result<Json<Tokens>> {
    payload.validate(&config.validation)?;
    if let Some(result) = query!(
        "SELECT name, email FROM accounts WHERE name LIKE ? or email LIKE ?",
        &payload.account,
        &payload.email
    )
    .fetch_optional(db.inner())
    .await
    .context("validation")?
    {
        if result.name.eq_ignore_ascii_case(&payload.account) {
            return Err(Error::validation("Account already exists"));
        }
        if result.email.eq_ignore_ascii_case(&payload.email) {
            return Err(Error::validation("Email already exists"));
        }
        panic!("Unknown validation error");
    }

    let id = query!(
        "INSERT INTO accounts (name, password, email, created) VALUES (?, ?, ?, ?)",
        &payload.account,
        &payload.password,
        &payload.email,
        time::now() as i64
    )
    .execute(db.inner())
    .await
    .context("account insert")?
    .last_insert_id() as i32;

    let (token, refresh_token) = jwt.register(&config.jwt, id)?;
    Ok(Json(Tokens {
        token,
        refresh_token,
    }))
}

#[openapi]
#[post("/login", data = "<payload>")]
async fn login(
    payload: Json<Login>,
    config: &State<Config>,
    db: &State<Pool<MySql>>,
    jwt: &State<jwt::Service>,
) -> Result<Json<Tokens>> {
    let id = account_id(&payload, db.inner()).await?;
    let (token, refresh_token) = jwt.register(&config.jwt, id)?;
    Ok(Json(Tokens {
        token,
        refresh_token,
    }))
}

#[openapi]
#[post("/refresh")]
async fn refresh_token(
    refresh_token: JwtRefreshId,
    config: &State<Config>,
    jwt: &State<jwt::Service>,
) -> Result<Json<Tokens>> {
    let token = jwt.refresh(&config.jwt, refresh_token.0)?;
    Ok(Json(Tokens {
        token,
        refresh_token: refresh_token.1,
    }))
}

#[openapi]
#[post("/logout")]
fn logout(refresh_token: JwtRefreshId, jwt: &State<jwt::Service>) -> Status {
    jwt.unregister_token(refresh_token.0);
    Status::Ok
}

#[openapi]
#[get("/")]
async fn account(aid: JwtAccountId, db: &State<Pool<MySql>>) -> Result<Json<Account>> {
    let premium_points = query!("SELECT premium_points FROM accounts WHERE id=?", &aid.0)
        .fetch_one(db.inner())
        .await
        .context("nindo")?
        .premium_points;
    let characters = query_as!(
        Character,
        r#"SELECT id, name, level, deleted AS "deleted:_" FROM players WHERE account_id=?"#,
        &aid.0
    )
    .fetch_all(db.inner())
    .await
    .context("players")?;
    Ok(Json(Account {
        characters,
        premium_points,
    }))
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Create {
    account: String,
    password: String,
    email: String,
}

impl Create {
    fn validate(&self, cfg: &config::Validation) -> Result<()> {
        validate_string(&self.account, "account", cfg, false)?;
        validate_string(&self.password, "password", cfg, true)?;
        validate_string(&self.email, "email", cfg, true)?;
        let regex = Regex::new(
            r"^([a-zA-Z0-9_+]([a-zA-Z0-9_+.]*[a-zA-Z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
        )
        .unwrap();
        if regex.is_match(&self.email) {
            Ok(())
        } else {
            Err(Error::validation("Invalid email format"))
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Login {
    account: String,
    password: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Tokens {
    token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Account {
    premium_points: i32,
    characters: Vec<Character>,
}

#[derive(Serialize, Deserialize, JsonSchema, FromRow)]
struct Character {
    id: i32,
    name: String,
    level: i32,
    deleted: bool,
}

async fn account_id(data: &Login, db: &Pool<MySql>) -> Result<i32> {
    query!(
        "SELECT id FROM accounts WHERE BINARY name=? AND BINARY password=?",
        &data.account,
        &data.password
    )
    .fetch_optional(db)
    .await
    .context("aid")?
    .map(|r| r.id)
    .ok_or(Error::status(Status::Unauthorized))
}

fn validate_string(
    value: &str,
    name: &'static str,
    cfg: &config::Validation,
    allow_punctuation: bool,
) -> Result<()> {
    if cfg.min_length > value.len() {
        Err(Error::validation(format!("{name} is too short")))
    } else if cfg.max_length < value.len() {
        Err(Error::validation(format!("{name} is too long")))
    } else if value.contains(|c: char| {
        !c.is_ascii_alphanumeric() && (!allow_punctuation || !c.is_ascii_punctuation())
    }) {
        Err(Error::validation(format!(
            "{name} contains invalid characters"
        )))
    } else {
        Ok(())
    }
}

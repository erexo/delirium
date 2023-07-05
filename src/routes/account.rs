use rocket::http::Status;
use rocket::serde::json::Json;

use rocket::{get, post};
use rocket::{Route, State};
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, MySql, Pool};

use crate::config::Config;
use crate::services::jwt;

use super::{JwtAccountId, JwtRefreshId};

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![login, refresh_token, logout, account]
}

#[openapi]
#[post("/login", data = "<payload>")]
async fn login(
    payload: Json<Login>,
    config: &State<Config>,
    db: &State<Pool<MySql>>,
    jwt: &State<jwt::Service>,
) -> Result<Json<Tokens>, Status> {
    let id = account_id(&payload, db.inner()).await?;
    let (token, refresh_token) = jwt.register(&config.jwt, id).expect("tokens");
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
) -> Result<Json<Tokens>, Status> {
    let token = jwt.refresh(&config.jwt, refresh_token.0).expect("token");
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
#[get("/account")]
async fn account(aid: JwtAccountId, db: &State<Pool<MySql>>) -> Json<Account> {
    let premium_points = query!("SELECT premium_points FROM accounts WHERE id=?", &aid.0)
        .fetch_one(db.inner())
        .await
        .expect("nindo")
        .premium_points;
    let players = query_as!(
        Player,
        "SELECT id, name, level FROM players WHERE account_id=?",
        &aid.0
    )
    .fetch_all(db.inner())
    .await
    .expect("players");
    Json(Account {
        players,
        premium_points,
    })
}

async fn account_id(data: &Login, db: &Pool<MySql>) -> Result<i32, Status> {
    let id = query!(
        "SELECT id FROM accounts WHERE BINARY name=? AND BINARY password=?",
        &data.account,
        &data.password
    )
    .fetch_one(db)
    .await;
    match id {
        Ok(o) => Ok(o.id),
        Err(err) => match err {
            sqlx::Error::RowNotFound => return Err(Status::Unauthorized),
            _ => panic!("{err}"),
        },
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
    players: Vec<Player>,
}

#[derive(Serialize, Deserialize, JsonSchema, FromRow)]
struct Player {
    id: i32,
    name: String,
    level: i32,
}
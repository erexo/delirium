use rocket::http::Status;
use rocket::serde::json::Json;

use rocket::{get, post};
use rocket::{Route, State};
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::config::Config;
use crate::services::jwt;

use super::JwtRefreshId;

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![login, refresh_token, logout]
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
#[get("/logout")]
fn logout(refresh_token: JwtRefreshId, jwt: &State<jwt::Service>) -> Status {
    jwt.unregister_token(refresh_token.0);
    Status::Ok
}

async fn account_id(data: &Login, db: &Pool<MySql>) -> Result<i64, Status> {
    let id: sqlx::Result<(i64,)> =
        sqlx::query_as("SELECT id FROM accounts WHERE BINARY name=? AND BINARY password=?")
            .bind(&data.account)
            .bind(&data.password)
            .fetch_one(db)
            .await;
    match id {
        Ok(o) => Ok(o.0),
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
pub struct Tokens {
    token: String,
    refresh_token: String,
}

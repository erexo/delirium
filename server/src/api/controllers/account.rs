use std::sync::Arc;

use crate::{
    api::jwt_bearer::{JwtAccountId, JwtRefreshId},
    services::jwt,
    utils::time,
};

use super::prelude::*;
use anyhow::Context;
use delirium_macros::Validation;
use poem::http::StatusCode;
use poem_openapi::{payload::Json, Object, OpenApi};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, MySql, Pool};

pub struct Api {
    db: Pool<MySql>,
    jwt: Arc<jwt::Service>,
}

pub fn api(db: &Pool<MySql>, jwt: &Arc<jwt::Service>) -> Api {
    Api {
        db: db.clone(),
        jwt: jwt.clone(),
    }
}

#[OpenApi(prefix_path = "/account", tag = "super::Tags::Account")]
impl Api {
    /// Create Account
    #[oai(path = "/", method = "put")]
    async fn create(&self, mut data: Json<Create>) -> Result<Json<Tokens>> {
        data.validate()?;
        if let Some(result) = query!(
            "SELECT name, email FROM accounts WHERE name LIKE ? or email LIKE ?",
            &data.account,
            &data.email
        )
        .fetch_optional(&self.db)
        .await
        .context("validation")?
        {
            if result.name.eq_ignore_ascii_case(&data.account) {
                return Err(AccountAlreadyExists.into());
            }
            if result.email.eq_ignore_ascii_case(&data.email) {
                return Err(EmailAlreadyExists.into());
            }
            panic!("Unknown validation error");
        }

        let id = query!(
            "INSERT INTO accounts (name, password, email, created) VALUES (?, ?, ?, ?)",
            &data.account,
            &data.password,
            &data.email,
            time::now() as i64
        )
        .execute(&self.db)
        .await
        .context("account insert")?
        .last_insert_id() as i32;

        let (account_token, refresh_token) = self.jwt.register(id)?;
        Ok(Json(Tokens {
            account_token,
            refresh_token,
        }))
    }

    /// Generate login tokens
    #[oai(path = "/login", method = "post")]
    async fn login(&self, data: Json<Login>) -> Result<Json<Tokens>> {
        let id = account_id(&data, &self.db).await?;
        let (account_token, refresh_token) = self.jwt.register(id)?;
        Ok(Json(Tokens {
            account_token,
            refresh_token,
        }))
    }

    /// Refresh token
    #[oai(path = "/refresh", method = "post")]
    async fn refresh_token(&self, data: JwtRefreshId) -> Result<Json<Tokens>> {
        let account_token = self.jwt.refresh(data.0.rid)?;
        Ok(Json(Tokens {
            account_token,
            refresh_token: data.0.refresh_token,
        }))
    }

    /// Discard refresh token
    #[oai(path = "/logout", method = "post")]
    async fn logout(&self, auth: JwtRefreshId) -> Result<()> {
        self.jwt.unregister_token(auth.0.rid);
        Ok(())
    }

    /// Get Account
    #[oai(path = "/", method = "get")]
    async fn account(&self, auth: JwtAccountId) -> Result<Json<Account>> {
        let premium_points = query!("SELECT premium_points FROM accounts WHERE id=?", &auth.0)
            .fetch_one(&self.db)
            .await
            .context("nindo")?
            .premium_points;
        let characters = query_as!(
            Character,
            r#"SELECT id, name, level, deleted AS "deleted:_" FROM players WHERE account_id=?"#,
            &auth.0
        )
        .fetch_all(&self.db)
        .await
        .context("players")?;
        Ok(Json(Account {
            characters,
            premium_points,
        }))
    }
}

#[derive(Object, Serialize, Deserialize, Validation)]
#[val(trim, length = "crate::config::field_length")]
struct Create {
    account: String,
    password: String,
    #[val(
        pattern = r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    )]
    email: String,
}

#[derive(Object, Serialize, Deserialize)]
struct Login {
    account: String,
    password: String,
}

#[derive(Object, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tokens {
    account_token: String,
    refresh_token: String,
}

#[derive(Object, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Account {
    premium_points: i32,
    characters: Vec<Character>,
}

#[derive(Object, Serialize, Deserialize, FromRow)]
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
    .ok_or(StatusCode::UNAUTHORIZED.into())
}

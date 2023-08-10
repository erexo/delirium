use crate::{api::jwt_bearer::JwtAccountId, config};

use super::prelude::*;
use anyhow::Context;
use poem_openapi::{payload::Json, OpenApi};
use sqlx::{query, MySql, Pool};
use tracing::info;

pub struct Api {
    db: Pool<MySql>,
}

pub fn api(db: &Pool<MySql>) -> Api {
    Api { db: db.clone() }
}

// name regex: ^(?:[a-zA-Z]{3,}\b(?:\s+[a-zA-Z]{3,}\b){0,2})?$
#[OpenApi(prefix_path = "/character", tag = "super::Tags::Character")]
impl Api {
    /// Delete Character
    #[oai(path = "/", method = "delete")]
    async fn get(&self, auth: JwtAccountId, id: Json<i32>) -> Result<()> {
        let record = query!(
            "SELECT id, account_id, level FROM players WHERE id=? AND NOT deleted",
            id.0
        )
        .fetch_optional(&self.db)
        .await
        .context("record")?;
        if let Some(record) = record {
            if record.account_id == auth.0 {
                let cfg = config::get();
                if record.level < cfg.character.insta_delete_below {
                    query!("DELETE FROM players WHERE id=?", record.id)
                        .execute(&self.db)
                        .await
                        .context("delete player")?;
                    info!("Deleted character '{}'", id.0);
                } else {
                    query!("UPDATE players SET deleted=1 WHERE id=?", record.id)
                        .execute(&self.db)
                        .await
                        .context("mark delete player")?;
                    info!("Marked character '{}' as deleted", id.0);
                }
            }
        };
        Ok(())
    }
}

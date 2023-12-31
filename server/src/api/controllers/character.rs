use crate::{api::jwt_bearer::JwtAccountId, config};

use super::prelude::*;
use anyhow::Context;
use delirium_macros::Validation;
use poem_openapi::{payload::Json, Object, OpenApi};
use sqlx::{query, MySql, Pool};
use tracing::info;

pub struct Api {
    db: Pool<MySql>,
}

pub fn api(db: &Pool<MySql>) -> Api {
    Api { db: db.clone() }
}

#[OpenApi(prefix_path = "/character", tag = "super::Tags::Character")]
impl Api {
    /// Create Character
    #[oai(path = "/", method = "put")]
    async fn create(
        &self,
        auth: JwtAccountId,
        mut data: Json<CreateCharacter>,
    ) -> Result<Json<i32>> {
        data.validate()?;
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data.world) {
            return Err(InvalidData.into());
        }
        let Some(voc) = cfg.character.new.vocations.get(&data.vocation) else {
            return Err(InvalidData.into());
        };

        if query!(
            "SELECT COUNT(*) count FROM players WHERE account_id = ?",
            &auth.0,
        )
        .fetch_optional(&self.db)
        .await
        .context("validation")?
        .is_some_and(|x| x.count >= cfg.account.max_characters as i64)
        {
            return Err(TooManyCharacters.into());
        }

        if query!(
            "SELECT name FROM players WHERE name LIKE ? LIMIT 1",
            &data.name,
        )
        .fetch_optional(&self.db)
        .await
        .context("validation")?
        .is_some()
        {
            return Err(PlayerAlreadyExists.into());
        }

        let cfg = &cfg.character.new;
        let id = query!("INSERT INTO players (name, world_id, account_id, vocation, health, healthmax, looktype, mana, manamax, soul, town_id, posx, posy, posz, cap) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        &data.name,
        &data.world,
        &auth.0,
        voc.vocation,
        cfg.health,
        cfg.health,
        voc.looktype,
        cfg.mana,
        cfg.mana,
        cfg.soul,
        cfg.town,
        cfg.pos_x,
        cfg.pos_y,
        cfg.pos_z,
        cfg.cap)
            .execute(&self.db)
            .await
            .context("player insert")?
            .last_insert_id() as i32;
        Ok(Json(id))
    }

    /// Delete Character
    #[oai(path = "/", method = "delete")]
    async fn delete(&self, auth: JwtAccountId, id: Json<i32>) -> Result<()> {
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

    /// Undelete Character
    #[oai(path = "/", method = "patch")]
    async fn undelete(&self, auth: JwtAccountId, id: Json<i32>) -> Result<()> {
        let record = query!(
            "SELECT id, account_id, level FROM players WHERE id=? AND deleted",
            id.0
        )
        .fetch_optional(&self.db)
        .await
        .context("record")?;
        if let Some(record) = record {
            if record.account_id == auth.0 {
                query!("UPDATE players SET deleted=0 WHERE id=?", record.id)
                    .execute(&self.db)
                    .await
                    .context("mark undelete player")?;
                info!("Undeleted character '{}'", id.0);
            }
        };
        Ok(())
    }
}

#[derive(Object, Validation)]
#[val(trim, ascii, length = "crate::config::field_length")]
struct CreateCharacter {
    #[val(to_title, pattern = r"^(?:[a-zA-Z]{3,}\b(?:\s+[a-zA-Z]{3,}\b){0,2})?$")]
    name: String,
    vocation: u32,
    world: u32,
}

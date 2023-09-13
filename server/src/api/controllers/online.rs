use crate::config;

use super::prelude::*;
use anyhow::Context;
use poem_openapi::{payload::Json, Object, OpenApi};
use sqlx::{query_as, FromRow, MySql, Pool};

pub struct Api {
    db: Pool<MySql>,
}

pub fn api(db: &Pool<MySql>) -> Api {
    Api { db: db.clone() }
}

#[OpenApi(prefix_path = "/online", tag = "super::Tags::Online")]
impl Api {
    /// Online Players
    #[oai(path = "/", method = "post")]
    async fn level(&self, data: Json<u32>) -> Result<Json<Vec<OnlinePlayer>>> {
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data) {
            return Err(InvalidData.into());
        }

        let characters = query_as!(
            OnlinePlayerRow,
            r#"SELECT id, name, level, vocation FROM players WHERE online = 1 AND group_id < 3 AND world_id = ? ORDER BY experience DESC"#,
            &data.0,
            )
            .fetch_all(&self.db)
            .await
            .context("level")?
            .into_iter()
            .map(|o| {
                let mut vocstr = "Unknown";
                'l: for (k, v) in &cfg.character.vocations {
                    for voc in v {
                        if &o.vocation == voc {
                            vocstr = k;
                            break 'l;
                        }
                    }
                }
                OnlinePlayer {
                    id: o.id,
                    name: o.name,
                    level: o.level,
                    vocation: vocstr.to_owned(),
                }})
        .collect::<Vec<_>>();
        Ok(Json(characters))
    }
}

#[derive(FromRow)]
struct OnlinePlayerRow {
    id: i32,
    name: String,
    level: u32,
    vocation: u32,
}

#[derive(Object)]
struct OnlinePlayer {
    id: i32,
    name: String,
    level: u32,
    vocation: String,
}

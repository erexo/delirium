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

#[OpenApi(prefix_path = "/deaths", tag = "super::Tags::Deaths")]
impl Api {
    /// Latest Deaths
    #[oai(path = "/", method = "post")]
    async fn deaths(&self, data: Json<u32>) -> Result<Json<Vec<Death>>> {
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data) {
            return Err(InvalidData.into());
        }

        let deaths = query_as!(
            DeathRow, 
            r#"SELECT pd.id, p.id AS player_id, p.name, p.level, pd.lost_experience, pd.date FROM player_deaths AS pd INNER JOIN players AS p ON pd.player_id = p.id WHERE p.world_id = ? ORDER BY pd.date DESC LIMIT 0, ?"#, 
            &data.0, 
            &cfg.deaths.page_count,
            )
            .fetch_all(&self.db)
            .await
            .context("deaths")?;

        let mut ret = Vec::new();

        for death in deaths {
            let killers = query_as!(
                KillerRow,
                r#"SELECT p.id, COALESCE(p.name, ek.name, '?') AS name FROM killers k LEFT JOIN environment_killers ek ON k.id = ek.kill_id LEFT JOIN player_killers pk ON k.id = pk.kill_id LEFT JOIN players p ON p.id = pk.player_id WHERE k.death_id = ? ORDER BY k.final_hit DESC, k.id ASC"#,
                death.id,
                )
                .fetch_all(&self.db)
                .await
                .context(format!("death {} killers", death.id))?
                .into_iter()
                .map(|o| {
                    let mut name = o.name;
                    if o.id.is_none() {
                        name = if let Some(stripped) = name.strip_prefix("a ") {
                            stripped.to_owned()
                        } else if let Some(stripped) = name.strip_prefix("an ") {
                            stripped.to_owned()
                        } else {
                            name
                        };
                    }
                    DeathKiller { id: o.id, name }
                })
                .collect::<Vec<_>>();

            ret.push(Death{ 
                id: death.player_id,
                name: death.name, 
                level: death.level,
                lost_experience: death.lost_experience,
                date: death.date,
                killers,
            });
        }

        Ok(Json(ret))
    }
}

#[derive(FromRow)]
struct DeathRow {
    id: i32,
    player_id: i32,
    name: String,
    level: u32,
    lost_experience: u64,
    date: u64,
}
 
#[derive(FromRow)]
struct KillerRow {
    id: Option<i32>,
    name: String,
}

#[derive(Object)]
struct Death {
    id: i32,
    name: String,
    level: u32,
    lost_experience: u64,
    date: u64,
    killers: Vec<DeathKiller>,
}

#[derive(Object)]
#[oai(skip_serializing_if_is_none = true)]
struct DeathKiller {
    id: Option<i32>,
    name: String,
}

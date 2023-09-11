use crate::config;

use super::prelude::*;
use anyhow::Context;
use poem_openapi::{payload::Json, Enum, Object, OpenApi};
use sqlx::{query_as, FromRow, MySql, Pool};

pub struct Api {
    db: Pool<MySql>,
}

pub fn api(db: &Pool<MySql>) -> Api {
    Api { db: db.clone() }
}

#[OpenApi(prefix_path = "/highscores", tag = "super::Tags::Highscores")]
impl Api {
    /// Level Highscores
    #[oai(path = "/level", method = "post")]
    async fn level(&self, data: Json<LevelHighscoresData>) -> Result<Json<Vec<LevelHighscores>>> {
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data.world) {
            return Err(InvalidData.into());
        }
        let count = cfg.highscores.page_count;
        let skip = count * data.page_number;

        let characters = query_as!(
            LevelHighscores,
            r#"SELECT id, name, level, experience FROM players WHERE group_id < 3 AND world_id = ? ORDER BY experience DESC LIMIT ?, ?"#,
            &data.world,
            &skip,
            &count,
        )
        .fetch_all(&self.db)
        .await
        .context("level")?;
        Ok(Json(characters))
    }

    /// Skill Highscores
    #[oai(path = "/skill", method = "post")]
    async fn skill(&self, data: Json<SkillHighscoresData>) -> Result<Json<Vec<SkillHighscores>>> {
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data.world) {
            return Err(InvalidData.into());
        }
        let count = cfg.highscores.page_count;
        let skip = count * data.page_number;

        let characters = if data.skill == Skill::Ninjutsu {
            query_as!(
                SkillHighscores,
                r#"SELECT id, name, maglevel AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY maglevel DESC LIMIT ?, ?"#,
                &data.world,
                &skip,
                &count,
            )
            .fetch_all(&self.db)
            .await
            .context("ninjutsu")?
        } else {
            query_as!(
                SkillHighscores,
                r#"SELECT p.id, p.name, s.value AS level FROM players p INNER JOIN player_skills s ON p.id = s.player_id WHERE p.group_id < 3 AND p.world_id = ? AND s.skillid = ? ORDER BY s.value DESC, s.count DESC LIMIT ?, ?"#,
                &data.world,
                data.skill as u32,
                &skip,
                &count,
            )
            .fetch_all(&self.db)
            .await
            .context("ninjutsu")?
        };
        Ok(Json(characters))
    }
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct LevelHighscoresData {
    world: u32,
    page_number: u32,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct SkillHighscoresData {
    skill: Skill,
    world: u32,
    page_number: u32,
}

#[derive(Enum, Copy, Clone, PartialEq)]
enum Skill {
    Fist = 0,
    Glove = 1,
    Sword = 2,
    Focus = 3,
    Distance = 4,
    Shielding = 5,
    Control = 6,
    Ninjutsu = 7,
}

#[derive(Object, FromRow)]
struct LevelHighscores {
    id: i32,
    name: String,
    level: i32,
    experience: i64,
}

#[derive(Object, FromRow)]
struct SkillHighscores {
    id: i32,
    name: String,
    level: u32,
}

use std::{collections::HashMap, fmt, sync::Mutex};

use crate::{config, utils::time};

use super::prelude::*;
use anyhow::Context;
use poem_openapi::{payload::Json, Enum, Object, OpenApi};
use sqlx::{query_as, FromRow, MySql, Pool};

pub struct Api {
    db: Pool<MySql>,
    vocation_cache: Mutex<HashMap<u32, VocationHighscoresCache>>,
}

pub fn api(db: &Pool<MySql>) -> Api {
    Api {
        db: db.clone(),
        vocation_cache: Mutex::new(HashMap::new()),
    }
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

        let characters = match data.skill {
            Skill::Fist => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_fist AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_fist DESC, skill_fist_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Club => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_club AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_club DESC, skill_club_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Sword => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_sword AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_sword DESC, skill_sword_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Axe => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_axe AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_axe DESC, skill_axe_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Distance => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_dist AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_dist DESC, skill_dist_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Shielding => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_shielding  AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_shielding DESC, skill_shielding_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Fishing => query_as!(
                SkillHighscores,
                r#"SELECT id, name, skill_fishing AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY skill_fishing DESC, skill_fishing_tries DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
            Skill::Magic => query_as!(
                SkillHighscores,
                r#"SELECT id, name, maglevel AS "level: u32" FROM players WHERE group_id < 3 AND world_id = ? ORDER BY maglevel DESC, manaspent DESC LIMIT ?, ?"#,
                &data.world, &skip, &count).fetch_all(&self.db).await,
        }.context(data.skill)?;

        Ok(Json(characters))
    }

    /// Vocation Highscores
    #[oai(path = "/vocation", method = "post")]
    async fn vocation(&self, data: Json<u32>) -> Result<Json<Vec<VocationHighscores>>> {
        let cfg = config::get();
        if !cfg.worlds.contains_key(&data.0) {
            return Err(InvalidData.into());
        }

        let cache_time = cfg.highscores.vocation_cache_time;
        if cache_time > 0 {
            let cache = self.vocation_cache.lock().unwrap();
            if let Some(cache) = cache.get(&data.0) {
                if cache.time + cache_time > time::now() {
                    return Ok(Json(cache.vocation_highscores.clone()));
                }
            }
        }

        let mut ret = Vec::new();
        for (name, vocations) in &cfg.character.vocations {
            let params = format!("?{}", ", ?".repeat(vocations.len() - 1));
            let query_str = format!(
                r#"SELECT id, name, level, ? AS vocation FROM players WHERE world_id = ? AND vocation IN ( {} ) ORDER BY experience DESC LIMIT 1"#,
                params
            );
            let mut query = sqlx::query_as::<_, VocationHighscores>(&query_str)
                .bind(&name)
                .bind(&data.0);
            for i in vocations {
                query = query.bind(i);
            }
            if let Some(row) = query.fetch_optional(&self.db).await.context("vocation")? {
                ret.push(row);
            }
        }

        if cache_time > 0 {
            let mut cache = self.vocation_cache.lock().unwrap();
            cache.insert(
                data.0,
                VocationHighscoresCache {
                    vocation_highscores: ret.clone(),
                    time: time::now(),
                },
            );
        }
        Ok(Json(ret))
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

#[derive(Debug, Enum, Copy, Clone, PartialEq)]
enum Skill {
    Fist = 0,
    Club = 1,
    Sword = 2,
    Axe = 3,
    Distance = 4,
    Shielding = 5,
    Fishing = 6,
    Magic = 7,
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Object, FromRow)]
struct LevelHighscores {
    id: i32,
    name: String,
    level: u32,
    experience: u64,
}

#[derive(Object, FromRow)]
struct SkillHighscores {
    id: i32,
    name: String,
    level: u32,
}

struct VocationHighscoresCache {
    vocation_highscores: Vec<VocationHighscores>,
    time: usize,
}

#[derive(Object, FromRow, Clone)]
struct VocationHighscores {
    id: i32,
    name: String,
    level: u32,
    vocation: String,
}

use rocket::{delete, http::Status, Route, State};
use rocket_okapi::{okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec};
use sqlx::{query, MySql, Pool};

use crate::config::Config;

use super::JwtAccountId;

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![delete]
}

#[openapi]
#[delete("/<name>")]
async fn delete(
    name: String,
    aid: JwtAccountId,
    cfg: &State<Config>,
    db: &State<Pool<MySql>>,
) -> Status {
    if let Some(record) = query!(
        "SELECT id, account_id, level FROM players WHERE name=? AND NOT deleted",
        name.trim()
    )
    .fetch_optional(db.inner())
    .await
    .expect("record")
    {
        if record.account_id != aid.0 {
            return Status::Unauthorized;
        }
        if record.level < cfg.character.insta_delete_below {
            query!("DELETE FROM players WHERE id=?", record.id)
                .execute(db.inner())
                .await
                .expect("delete player");
        } else {
            query!("UPDATE players SET deleted=1 WHERE id=?", record.id)
                .execute(db.inner())
                .await
                .expect("mark delete player");
        }
    };
    Status::Accepted
}

use std::time::Instant;

use rocket::{get, serde::json::Json, Route, State};
use rocket_okapi::{okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec};
use sqlx::{MySql, Pool};

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![hi]
}

#[openapi]
#[get("/id/<name>")]
async fn hi(name: String, db: &State<Pool<MySql>>) -> Json<(i64, u128)> {
    let n = Instant::now();
    let id: (i64,) = sqlx::query_as("SELECT id from players WHERE name LIKE ?")
        .bind(name)
        .fetch_one(db.inner())
        .await
        .unwrap();

    Json((id.0, n.elapsed().as_millis()))
}

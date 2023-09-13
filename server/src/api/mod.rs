use std::sync::Arc;

use poem::{
    http::StatusCode, middleware::CatchPanic, Endpoint, EndpointExt, IntoEndpoint, Middleware,
    Route,
};
use poem_openapi::OpenApiService;
use sqlx::{MySql, Pool};
use tracing::error;

use crate::{config, services::jwt};

pub mod controllers;
pub mod jwt_bearer;
pub mod trace_error;
pub mod validation_error;

pub fn routes(db: &Pool<MySql>, jwt: jwt::Service) -> impl IntoEndpoint {
    let jwt = &Arc::new(jwt);
    use controllers::*;
    let controllers = (
        validation::Api,
        account::api(db, jwt),
        character::api(db),
        highscores::api(db),
        online::api(db),
    );
    let api = OpenApiService::new(controllers, &config::get().api.name, "1.0");
    let docs = api.swagger_ui();
    Route::new()
        .nest("/", api)
        .nest("/swagger", docs)
        .with(catch_panic())
        .with(trace_error::TraceError)
}

fn catch_panic<E: Endpoint>() -> impl Middleware<E> {
    CatchPanic::new().with_handler(|err| {
        error!("{:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

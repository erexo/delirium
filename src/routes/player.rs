use rocket::{get, Route};
use rocket_okapi::{okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec};

pub(super) fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![hi]
}

#[openapi]
#[get("/hi")]
fn hi() -> &'static str {
    "hi!"
}

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::{
        merge::marge_spec_list,
        openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData},
    },
    request::{OpenApiFromRequest, RequestHeaderInput},
    settings::OpenApiSettings,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use serde::de::DeserializeOwned;

use crate::{
    config::{self, Config},
    services::jwt::{Claims, RefreshClaims},
};

mod account;
mod highscores;
mod player;

pub fn attach() -> AdHoc {
    AdHoc::on_ignite("Manage routes", |mut rocket| async {
        let mut apis = Vec::new();
        for (path, (routes, openapi)) in [
            ("/account", account::routes()),
            ("/highscores", highscores::routes()),
            ("/player", player::routes()),
        ] {
            rocket = rocket.mount(path, routes);
            apis.push((path, openapi));
        }
        if let Some(cfg) = rocket.state::<Config>() {
            if cfg.debug.swagger {
                rocket = rocket
                    .mount(
                        "/",
                        vec![rocket_okapi::get_openapi_route(
                            marge_spec_list(&apis).expect("spec"),
                            &OpenApiSettings::default(),
                        )],
                    )
                    .mount(
                        "/swagger",
                        make_swagger_ui(&SwaggerUIConfig {
                            url: "../openapi.json".to_owned(),
                            ..Default::default()
                        }),
                    )
            }
        }
        rocket
    })
}

struct JwtAccountId(i32);
struct JwtRefreshId(u128, String);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for JwtAccountId {
    type Error = String;
    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        match bearer(request) {
            Ok(token) => {
                let cfg = request.guard::<&rocket::State<Config>>().await.unwrap();
                match validate::<Claims>(token, &cfg.jwt) {
                    Ok(data) => Outcome::Success(JwtAccountId(data.aid())),
                    Err(err) => Outcome::Failure((Status::Unauthorized, err)),
                }
            }
            Err(msg) => Outcome::Failure((Status::BadRequest, msg.to_owned())),
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for JwtAccountId {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        internal_from_request_input(_gen, _name, _required)
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for JwtRefreshId {
    type Error = String;
    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        match bearer(request) {
            Ok(token) => {
                let cfg = request.guard::<&rocket::State<Config>>().await.unwrap();
                match validate::<RefreshClaims>(token, &cfg.jwt) {
                    Ok(data) => Outcome::Success(JwtRefreshId(data.rid(), token.to_owned())),
                    Err(err) => Outcome::Failure((Status::Unauthorized, err)),
                }
            }
            Err(msg) => Outcome::Failure((Status::BadRequest, msg.to_owned())),
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for JwtRefreshId {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        internal_from_request_input(_gen, _name, _required)
    }
}

fn bearer<'a>(request: &'a Request<'_>) -> Result<&'a str, &'a str> {
    match request.headers().get_one("Authorization") {
        Some(token) => {
            const PREFIX: &str = "Bearer ";
            if token.starts_with(PREFIX) {
                Ok(token.trim_start_matches(PREFIX))
            } else {
                Err("Invalid `Authorization` header.")
            }
        }
        None => Err("Missing `Authorization` header."),
    }
}

fn validate<T: DeserializeOwned>(token: &str, jwt: &config::Jwt) -> Result<T, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.sub = jwt.subject.clone();
    if let Some(ref audience) = jwt.audience {
        validation.set_audience(&[audience.clone()]);
    }
    match decode::<T>(
        token,
        &DecodingKey::from_secret(jwt.secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => Ok(data.claims),
        Err(err) => Err(err.to_string()),
    }
}

fn internal_from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
) -> rocket_okapi::Result<RequestHeaderInput> {
    let security_scheme = SecurityScheme {
        description: Some("Requires an JWT Bearer token to access.".to_owned()),
        data: SecuritySchemeData::Http {
            scheme: "bearer".to_owned(),
            bearer_format: Some("bearer".to_owned()),
        },
        extensions: Object::default(),
    };
    let mut security_req = SecurityRequirement::new();
    security_req.insert("HttpAuth".to_owned(), Vec::new());
    Ok(RequestHeaderInput::Security(
        "HttpAuth".to_owned(),
        security_scheme,
        security_req,
    ))
}

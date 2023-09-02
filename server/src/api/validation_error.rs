use delirium_macros::{DisplayUpperSnake, JsonParameters, ResponseEnum};
use poem::{error::ResponseError, http::StatusCode, IntoResponse, Response};
use poem_openapi::{payload::Json, Object};
use serde_json::Value;

#[derive(Debug, DisplayUpperSnake, ResponseEnum, JsonParameters, thiserror::Error)]
pub enum ValidationError {
    Unknown,
    MinLength { field: &'static str, min: usize },
    MaxLength { field: &'static str, max: usize },
    Ascii { field: &'static str },
    Alphanumeric { field: &'static str },
    Pattern { field: &'static str, value: String },
    InvalidData,
    // EntityNotExists(&'static str),
    AccountAlreadyExists,
    EmailAlreadyExists,
    IndistinctPasswords,
    InvalidCurrentPassword,
    TooManyCharacters,
    PlayerAlreadyExists,
}

#[derive(Object)]
#[oai(rename_all = "camelCase", skip_serializing_if_is_none = true)]
pub struct ValidationErrorBody {
    code: String,
    parameters: Option<Vec<Option<Value>>>,
}

impl ResponseError for ValidationError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn as_response(&self) -> Response {
        Json(ValidationErrorBody {
            code: self.to_string(),
            parameters: self.parameters(),
        })
        .with_status(self.status())
        .into_response()
    }
}

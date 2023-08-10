use poem_openapi::OpenApi;

use crate::api::validation_error::ValidationError;

pub struct Api;

#[OpenApi(tag = "super::Tags::Validation")]
impl Api {
    #[oai(path = "/v", method = "get")]
    async fn v(&self) -> ValidationError {
        ValidationError::Unknown
    }
}

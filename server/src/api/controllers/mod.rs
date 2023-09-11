pub(super) mod account;
pub(super) mod character;
pub(super) mod highscores;
pub(super) mod validation;

mod prelude {
    pub use crate::api::validation_error::ValidationError::*;
    pub use poem::Result;
}

#[derive(poem_openapi::Tags)]
pub enum Tags {
    Account,
    Character,
    Highscores,
    Validation,
}

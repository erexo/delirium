extern crate proc_macro;

mod display_upper_snake;
mod json_parameters;
mod response_enum;
mod validation;

use display_upper_snake::derive_display_upper_snake_impl;
use json_parameters::derive_json_parameters_impl;
use proc_macro::TokenStream;
use response_enum::derive_response_enum_impl;
use validation::derive_validation_impl;

#[proc_macro_derive(DisplayUpperSnake)]
pub fn derive_display_upper_snake(input: TokenStream) -> TokenStream {
    TokenStream::from(
        derive_display_upper_snake_impl(input.into()).unwrap_or_else(|err| err.to_compile_error()),
    )
}

#[proc_macro_derive(ResponseEnum)]
pub fn derive_response_enum(input: TokenStream) -> TokenStream {
    TokenStream::from(
        derive_response_enum_impl(input.into()).unwrap_or_else(|err| err.to_compile_error()),
    )
}

#[proc_macro_derive(JsonParameters)]
pub fn derive_json_parameters(input: TokenStream) -> TokenStream {
    TokenStream::from(
        derive_json_parameters_impl(input.into()).unwrap_or_else(|err| err.to_compile_error()),
    )
}

#[proc_macro_derive(Validation, attributes(val))]
pub fn derive_validation(input: TokenStream) -> TokenStream {
    TokenStream::from(
        derive_validation_impl(input.into()).unwrap_or_else(|err| err.to_compile_error()),
    )
}

use darling::{
    ast::Data,
    util::{Ignored, SpannedValue},
    FromDeriveInput, FromField,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{DeriveInput, Error, Ident, Path, Type, TypePath};

#[derive(FromField)]
#[darling(attributes(val))]
struct ValidationField {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    trim: bool,
    #[darling(default)]
    length: Option<Path>,
    #[darling(default)]
    pattern: Option<SpannedValue<String>>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(val))]
struct ValidationInput {
    ident: Ident,
    data: Data<Ignored, ValidationField>,

    #[darling(default)]
    trim: bool,
    #[darling(default)]
    length: Option<Path>,
}

pub fn derive_validation_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input).unwrap();
    let input = ValidationInput::from_derive_input(&input)?;

    let ident = &input.ident;
    let args = match &input.data {
        Data::Struct(args) => args,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "Validation can only be applied to struct",
            ))
        }
    };
    let string_type = get_type("String");

    let mut trims = Vec::new();
    let mut lengths = Vec::new();
    let mut patterns = Vec::new();
    for field in &args.fields {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(ident, "All fields must be named"))?;
        let ident_str = ident.to_string();
        if field.trim || input.trim {
            if field.ty == string_type {
                trims.push(quote! {
                    trim_in_place::TrimInPlace::trim_in_place(&mut self.#ident);
                })
            } else if field.trim {
                return Err(Error::new_spanned(
                    ident,
                    "Trim attr may only be applied on String field",
                ));
            }
        }

        if let Some(length) = field.length.as_ref().or(input.length.as_ref()) {
            if field.ty == string_type {
                lengths.push(quote!{
                    let (min, max) = #length();
                    if min > 0 && self.#ident.len() < min {
                        return Err(crate::api::validation_error::ValidationError::MinLength { field: #ident_str, min }.into());
                    }
                    if max > 0 && self.#ident.len() > max {
                        return Err(crate::api::validation_error::ValidationError::MaxLength { field: #ident_str, max }.into());
                    }
                });
            } else if field.length.is_some() {
                return Err(Error::new_spanned(
                    ident,
                    "Minmax attr may only be applied on String field",
                ));
            }
        }

        if let Some(pattern) = &field.pattern {
            if let Err(err) = Regex::new(pattern) {
                return Err(Error::new_spanned(
                    ident,
                    format!("Pattern attr contains invalid regular expression. {err}"),
                ));
            }
            let pattern = &**pattern;
            patterns.push(quote! {
                if !anyhow::Context::context(regex::Regex::new(#pattern), "Validation Pattern")?.is_match(&self.#ident) {
                    return Err(crate::api::validation_error::ValidationError::Pattern { field: #ident_str, value: self.#ident.to_string() }.into());
                }
            });
        }
    }
    Ok(quote! {
        impl #ident {
            pub fn validate(&mut self) -> poem::Result<()> {
                #(#trims)*
                #(#lengths)*
                #(#patterns)*
                Ok(())
            }
        }
    })
}

fn get_type(name: &str) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: Path::from(Ident::new(name, Span::call_site())),
    })
}

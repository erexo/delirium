use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Type};

pub fn derive_response_enum_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input).unwrap();

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "Input must be an enum")),
    };

    let ident = input.ident;
    let ident_str = ident.to_string();
    let parameters = variants
        .iter()
        .map(|variant| {
            let code = variant.ident.to_string().to_case(Case::UpperSnake);
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let params = fields
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().map_or("?".to_owned(), |i| i.to_string()))
                        .collect::<Vec<_>>()
                        .join(",");
                    quote! { serde_json::Value::String(format!("{}({})", #code, #params)), }
                }
                syn::Fields::Unnamed(fields) => {
                    let params = fields
                        .unnamed
                        .iter()
                        .map(|f| {
                            let mut ty = &f.ty;
                            if let Type::Reference(r) = ty {
                                ty = &r.elem;
                            }
                            if let Type::Path(p) = ty {
                                if !p.path.segments.is_empty() {
                                    let mut type_string = p.path.segments[0].ident.to_string();
                                    if &type_string == "str" {
                                        type_string = "String".to_string();
                                    }
                                    return type_string;
                                }
                            }
                            "?".to_owned()
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    quote! { serde_json::Value::String(format!("{}({})", #code, #params)), }
                }
                syn::Fields::Unit => {
                    quote! { serde_json::Value::String(#code.to_owned()), }
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(quote! {
        impl poem_openapi::ApiResponse for #ident {
            fn meta() -> poem_openapi::registry::MetaResponses {
                poem_openapi::registry::MetaResponses {
                    responses: vec![poem_openapi::registry::MetaResponse {
                        description: concat!("Marker endpoint to generate ", #ident_str, " Schema"),
                        status: Some(200),
                        content: vec![poem_openapi::registry::MetaMediaType {
                            content_type: "text/plain; charset=utf-8",
                            schema: poem_openapi::registry::MetaSchemaRef::Reference(#ident_str.to_owned()),
                        }],
                        headers: vec![],
                    }],
                }
            }

            fn register(registry: &mut poem_openapi::registry::Registry) {
                registry.create_schema::<Self, _>("ValidationError".to_owned(), |_| {
                    poem_openapi::registry::MetaSchema {
                        description: Some("Available codes with parameters"),
                        enum_items: vec![
                            #(#parameters)*
                        ],
                        ..poem_openapi::registry::MetaSchema::new("string")
                    }
                });
            }
        }

        impl IntoResponse for #ident {
            fn into_response(self) -> Response {
                poem::http::StatusCode::NOT_FOUND.into_response()
            }
        }
    })
}

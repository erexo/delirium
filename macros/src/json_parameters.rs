use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Data, DeriveInput, Fields};

pub fn derive_json_parameters_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input).unwrap();

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "Input must be an enum")),
    };

    let ident = input.ident;
    let parameters = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref())
                        .collect::<Vec<_>>();
                    quote_spanned! { variant.span() =>
                        Self::#ident {#(#fields,)*} => Some(vec![#(#fields.to_json(),)*]),
                    }
                }
                Fields::Unnamed(fields) => {
                    let fields: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| format_ident!("f{i}"))
                        .collect();
                    quote_spanned! { variant.span() =>
                        Self::#ident (#(#fields,)*) => Some(vec![#(#fields.to_json(),)*]),
                    }
                }
                Fields::Unit => quote! { Self::#ident => None, },
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl #ident {
            pub fn parameters(&self) -> Option<Vec<Option<serde_json::Value>>> {
                use poem_openapi::types::ToJSON;
                match self {
                    #(#parameters)*
                }
            }
        }
    })
}

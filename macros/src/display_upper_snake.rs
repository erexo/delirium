use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive_display_upper_snake_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input).unwrap();

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "Input must be an enum")),
    };

    let ident = input.ident;
    let codes = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let code = ident.to_string().to_case(Case::UpperSnake);
            match variant.fields {
                Fields::Named(_) => quote! { Self::#ident {..} => #code, },
                Fields::Unnamed(_) => quote! { Self::#ident (..) => #code, },
                Fields::Unit => quote! { Self::#ident => #code, },
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    #(#codes)*
                })
            }
        }
    })
}

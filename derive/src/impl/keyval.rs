use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn key(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    if attr.to_string().trim() != "" {
        panic!("#[key] does not accept any arguments");
    }

    let input = parse_macro_input!(item as DeriveInput)?;

    let derives = [
        quote! { ::core::fmt::Debug },
        quote! { ::core::clone::Clone },
        quote! { ::core::cmp::PartialEq },
        quote! { ::core::cmp::Eq },
        quote! { ::core::cmp::PartialOrd },
        quote! { ::core::cmp::Ord },
        quote! { ::core::hash::Hash },
    ];

    let expanded = quote! {
        #[derive(#(#derives),*)]
        #input
    };

    Ok(TokenStream::from(expanded))
}

pub fn val(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    if attr.to_string().trim() != "" {
        panic!("#[val] does not accept any arguments");
    }

    let input = parse_macro_input!(item as DeriveInput)?;

    let derives = [
        quote! { ::core::fmt::Debug },
        #[cfg(feature = "rkyv")]
        quote! { ::rkyv::Archive },
        #[cfg(feature = "rkyv")]
        quote! { ::rkyv::Serialize },
        #[cfg(feature = "rkyv")]
        quote! { ::rkyv::Deserialize },
        #[cfg(feature = "serde")]
        quote! { ::serde::Serialize },
        #[cfg(feature = "serde")]
        quote! { ::serde::Deserialize },
    ];

    let expanded = quote! {
        #[derive(#(#derives),*)]
        #input
    };

    Ok(TokenStream::from(expanded))
}

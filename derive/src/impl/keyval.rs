use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn key(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    if attr.to_string().trim() != "" {
        return Err(TokenStream::from(
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[key] does not accept any arguments",
            )
            .to_compile_error(),
        ));
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
        return Err(TokenStream::from(
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[val] does not accept any arguments",
            )
            .to_compile_error(),
        ));
    }

    let input = parse_macro_input!(item as DeriveInput)?;

    let derives = [
        quote! { ::core::fmt::Debug },
        #[cfg(feature = "rkyv")]
        quote! { ::data_classes::deps::rkyv::Archive },
        #[cfg(feature = "rkyv")]
        quote! { ::data_classes::deps::rkyv::Serialize },
        #[cfg(feature = "rkyv")]
        quote! { ::data_classes::deps::rkyv::Deserialize },
        #[cfg(feature = "serde")]
        quote! { ::data_classes::deps::serde::Serialize },
        #[cfg(feature = "serde")]
        quote! { ::data_classes::deps::serde::Deserialize },
    ];

    let expanded = quote! {
        #[derive(#(#derives),*)]
        #input
    };

    Ok(TokenStream::from(expanded))
}

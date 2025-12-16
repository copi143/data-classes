use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{DeriveInput, parse_macro_input};

pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = attr.to_string().to_ascii_lowercase();
    let attr = attr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty());
    let mut attr = attr.collect::<HashSet<_>>();

    let input = parse_macro_input!(item as DeriveInput);
    let ident = &input.ident;

    let mut reprs = Vec::new();
    let mut derives = Vec::new();
    let mut impls = Vec::new();

    derives.extend_from_slice(&[
        quote! { ::core::fmt::Debug },
        quote! { ::core::clone::Clone },
        quote! { ::core::cmp::PartialEq },
        quote! { ::core::cmp::Eq },
        quote! { ::core::cmp::PartialOrd },
        quote! { ::core::cmp::Ord },
        quote! { ::core::hash::Hash },
    ]);

    if attr.remove("raw") {
        reprs.push(quote! { C });
    }

    if attr.remove("packed") {
        reprs.push(quote! { packed });
    }

    if attr.remove("transparent") {
        reprs.push(quote! { transparent });
    }

    if attr.remove("u8") {
        reprs.push(quote! { u8 });
    }

    if attr.remove("u16") {
        reprs.push(quote! { u16 });
    }

    if attr.remove("u32") {
        reprs.push(quote! { u32 });
    }

    if attr.remove("u64") {
        reprs.push(quote! { u64 });
    }

    if attr.remove("usize") {
        reprs.push(quote! { usize });
    }

    if attr.remove("i8") {
        reprs.push(quote! { i8 });
    }

    if attr.remove("i16") {
        reprs.push(quote! { i16 });
    }

    if attr.remove("i32") {
        reprs.push(quote! { i32 });
    }

    if attr.remove("i64") {
        reprs.push(quote! { i64 });
    }

    if attr.remove("isize") {
        reprs.push(quote! { isize });
    }

    if attr.remove("default") {
        derives.push(quote! { ::core::default::Default });
    }

    if attr.remove("copy") {
        derives.push(quote! { ::core::marker::Copy });
    }

    #[cfg(feature = "rkyv")]
    if attr.remove("rkyv") {
        derives.push(quote! { ::rkyv::Archive });
        derives.push(quote! { ::rkyv::Serialize });
        derives.push(quote! { ::rkyv::Deserialize });
    }

    #[cfg(feature = "serde")]
    if attr.remove("serde") {
        derives.push(quote! { ::serde::Serialize });
        derives.push(quote! { ::serde::Deserialize });
    }

    #[cfg(feature = "bytemuck")]
    if attr.remove("pod") {
        derives.push(quote! { ::bytemuck::Pod });
    }

    #[cfg(feature = "bytemuck")]
    if attr.remove("zeroable") {
        derives.push(quote! { ::bytemuck::Zeroable });
    }

    if attr.remove("new-default") {
        derives.push(quote! { ::core::default::Default });
        impls.push(quote! {
            impl #ident {
                pub fn new() -> Self {
                    Self::default()
                }
            }
        });
    }

    if attr.remove("new") {
        let fields: Vec<(&syn::Ident, &syn::Type)> = match &input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(fields),
                ..
            }) => fields
                .named
                .iter()
                .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
                .collect::<Vec<_>>(),
            _ => panic!("#[data(new)] can only be applied to structs with named fields"),
        };
        let field_names = fields.iter().map(|(name, _)| name).collect::<Vec<_>>();
        let field_types = fields.iter().map(|(_, ty)| ty).collect::<Vec<_>>();
        impls.push(quote! {
            impl #ident {
                pub fn new(#(#field_names: #field_types),*) -> Self {
                    Self {
                        #(#field_names),*
                    }
                }
            }
        });
    }

    if !attr.is_empty() {
        panic!("Unsupported attribute for #[data]: {:?}", attr);
    }

    let expanded = quote! {
        #[repr(#(#reprs),*)]
        #[derive(#(#derives),*)]
        #input
        #(#impls)*
    };

    expanded.into()
}

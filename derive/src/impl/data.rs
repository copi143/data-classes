use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::util::parse_attr_tree::AttrArgs;

pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = parse_macro_input!(attr as AttrArgs);

    let input = parse_macro_input!(item as DeriveInput);
    let ident = &input.ident;

    let mut reprs = Vec::new();
    let mut derives = Vec::new();
    let mut impls = Vec::new();
    let mut rkyv_derives = Vec::new();

    derives.extend_from_slice(&[
        quote! { ::core::fmt::Debug },
        quote! { ::core::clone::Clone },
        quote! { ::core::cmp::PartialEq },
        quote! { ::core::cmp::Eq },
        quote! { ::core::cmp::PartialOrd },
        quote! { ::core::cmp::Ord },
        quote! { ::core::hash::Hash },
    ]);

    rkyv_derives.extend_from_slice(&[
        quote! { ::core::fmt::Debug },
        quote! { ::core::cmp::PartialEq },
        quote! { ::core::cmp::Eq },
        quote! { ::core::cmp::PartialOrd },
        quote! { ::core::cmp::Ord },
        quote! { ::core::hash::Hash },
    ]);

    macro_rules! repr_with_no_args {
        ($name:expr, $quote:expr) => {
            if let Some(args) = attr.remove($name) {
                reprs.push($quote);
                if !args.is_empty() {
                    panic!(concat!(
                        "#[data(",
                        $name,
                        ")] does not accept any arguments"
                    ));
                }
            }
        };
    }

    repr_with_no_args!("raw", quote! { C });
    repr_with_no_args!("packed", quote! { packed });
    repr_with_no_args!("transparent", quote! { transparent });

    repr_with_no_args!("u8", quote! { u8 });
    repr_with_no_args!("u16", quote! { u16 });
    repr_with_no_args!("u32", quote! { u32 });
    repr_with_no_args!("u64", quote! { u64 });
    repr_with_no_args!("usize", quote! { usize });
    repr_with_no_args!("i8", quote! { i8 });
    repr_with_no_args!("i16", quote! { i16 });
    repr_with_no_args!("i32", quote! { i32 });
    repr_with_no_args!("i64", quote! { i64 });
    repr_with_no_args!("isize", quote! { isize });

    if let Some(args) = attr.remove("default") {
        derives.push(quote! { ::core::default::Default });
        if !args.is_empty() {
            panic!("#[data(default)] does not accept any arguments");
        }
    }

    if let Some(args) = attr.remove("copy") {
        derives.push(quote! { ::core::marker::Copy });
        if !args.is_empty() {
            panic!("#[data(copy)] does not accept any arguments");
        }
    }

    #[cfg(not(feature = "rkyv"))]
    let mut rkyv: Option<AttrArgs> = None;
    #[cfg(feature = "rkyv")]
    let mut rkyv = attr.remove("rkyv");

    #[cfg(feature = "rkyv")]
    if rkyv.is_some() {
        derives.push(quote! { ::rkyv::Archive });
        derives.push(quote! { ::rkyv::Serialize });
        derives.push(quote! { ::rkyv::Deserialize });
    }

    #[cfg(feature = "serde")]
    if let Some(args) = attr.remove("serde") {
        derives.push(quote! { ::serde::Serialize });
        derives.push(quote! { ::serde::Deserialize });
        if !args.is_empty() {
            panic!("#[data(serde)] does not accept any arguments");
        }
    }

    #[cfg(feature = "bytemuck")]
    if let Some(args) = attr.remove("pod") {
        derives.push(quote! { ::bytemuck::Pod });
        if !args.is_empty() {
            panic!("#[data(pod)] does not accept any arguments");
        }
    }

    #[cfg(feature = "bytemuck")]
    if let Some(args) = attr.remove("zeroable") {
        derives.push(quote! { ::bytemuck::Zeroable });
        if !args.is_empty() {
            panic!("#[data(zeroable)] does not accept any arguments");
        }
    }

    if let Some(args) = attr.remove("debug-display") {
        impls.push(quote! {
            impl ::core::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        });
        if !args.is_empty() {
            panic!("#[data(debug-display)] does not accept any arguments");
        }
    }

    if let Some(args) = attr.remove("new-default") {
        derives.push(quote! { ::core::default::Default });
        impls.push(quote! {
            impl #ident {
                pub fn new() -> Self {
                    Self::default()
                }
            }
        });
        if !args.is_empty() {
            panic!("#[data(new-default)] does not accept any arguments");
        }
    }

    if let Some(args) = attr.remove("new") {
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
        if !args.is_empty() {
            panic!("#[data(new)] does not accept any arguments");
        }
    }

    if !attr.is_empty() {
        panic!("Unsupported attribute for #[data]: {attr}");
    }

    let rkyv_derives = if rkyv.is_some() {
        quote! { #[rkyv(derive(#(#rkyv_derives),*))] }
    } else {
        quote! {}
    };

    let rkyv_compares = if rkyv.is_some()
        && let Some(args) = rkyv.as_mut().unwrap().remove("cmp")
    {
        if !args.is_empty() {
            panic!("#[data(rkyv(cmp))] does not accept any arguments");
        }
        quote! { #[rkyv(compare(PartialEq, PartialOrd))] }
    } else {
        quote! {}
    };

    let rkyv_bounds = if rkyv.is_some()
        && let Some(args) = rkyv.as_mut().unwrap().remove("omit-bounds")
    {
        if !args.is_empty() {
            panic!("#[data(rkyv(omit-bounds))] does not accept any arguments");
        }
        quote! {
            #[rkyv(serialize_bounds(__S: ::rkyv::ser::Writer + ::rkyv::ser::Allocator, __S::Error: ::rkyv::rancor::Source))]
            #[rkyv(deserialize_bounds(__D::Error: ::rkyv::rancor::Source))]
            #[rkyv(bytecheck(bounds(__C: ::rkyv::validation::ArchiveContext)))]
        }
    } else {
        quote! {}
    };

    if let Some(rkyv) = rkyv
        && !rkyv.is_empty()
    {
        panic!("Unsupported attribute for #[data(rkyv)]: {rkyv}");
    }

    let expanded = quote! {
        #[repr(#(#reprs),*)]
        #[derive(#(#derives),*)]
        #rkyv_derives
        #rkyv_compares
        #rkyv_bounds
        #input
        #(#impls)*
    };

    expanded.into()
}

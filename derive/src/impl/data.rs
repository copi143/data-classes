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

    #[cfg(not(feature = "rkyv"))]
    let (use_rkyv, use_rkyv_bounds) = (false, false);
    #[cfg(feature = "rkyv")]
    let (use_rkyv, use_rkyv_bounds) = if attr.remove("rkyv") {
        if attr.remove("rkyv-with-bounds") {
            panic!("Cannot use both 'rkyv' and 'rkyv-with-bounds' attributes together");
        } else {
            (true, false)
        }
    } else if attr.remove("rkyv-with-bounds") {
        (true, true)
    } else {
        (false, false)
    };

    #[cfg(feature = "rkyv")]
    if use_rkyv {
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

    if attr.remove("debug-display") {
        impls.push(quote! {
            impl ::core::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        });
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

    let rkyv_derives = if use_rkyv {
        quote! { #[rkyv(derive(#(#rkyv_derives),*))] }
    } else {
        quote! {}
    };

    let rkyv_compares = if use_rkyv {
        quote! { #[rkyv(compare(PartialEq, PartialOrd))] }
    } else {
        quote! {}
    };

    let rkyv_bounds = if use_rkyv_bounds {
        quote! {
            #[rkyv(serialize_bounds(__S: ::rkyv::ser::Writer + ::rkyv::ser::Allocator, __S::Error: ::rkyv::rancor::Source))]
            #[rkyv(deserialize_bounds(__D::Error: ::rkyv::rancor::Source))]
            #[rkyv(bytecheck(bounds(__C: ::rkyv::validation::ArchiveContext)))]
        }
    } else {
        quote! {}
    };

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

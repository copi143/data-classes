use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::util::{
    fields_attr::{Enabled as FieldsAttrEnabledFeatures, FieldsAttr},
    parse_attr_tree::AttrArgs,
};

pub fn fields_list(input: DeriveInput) -> Option<Vec<syn::Ident>> {
    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => Some(match fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().clone())
                .collect(),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| syn::Ident::new(&format!("{i}"), proc_macro2::Span::call_site()))
                .collect(),
            syn::Fields::Unit => vec![],
        }),
        _ => None,
    }
}

pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let ident = &input.ident;

    let mut attr = {
        let mut attrs = Vec::new();
        for attr in std::mem::take(&mut input.attrs) {
            if attr.path().is_ident("data") {
                let syn::Meta::List(attr) = attr.meta else {
                    panic!("#[data(...)] attribute must be in list form");
                };
                attrs.push(attr.tokens.into());
            } else {
                input.attrs.push(attr);
            }
        }
        let mut args = parse_macro_input!(attr as AttrArgs);
        for attr in attrs {
            args.combine(parse_macro_input!(attr as AttrArgs));
        }
        args
    };

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

    #[allow(unused_variables)]
    let repr_c = attr.get("raw").is_some();
    #[allow(unused_variables)]
    let repr_transparent = attr.get("transparent").is_some();

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

    let enabled_attr = &FieldsAttrEnabledFeatures {
        default: attr.get("default").is_some(),
        new: attr.get("new").is_some(),
        add_comment_on_changed: true,
    };
    let fields_attr = match &mut input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => Some(FieldsAttr::parse(ident, &mut fields.named, enabled_attr)),
        _ => None,
    };

    if let Some(args) = attr.remove("default") {
        if let Some(attrs) = &fields_attr {
            if attrs.default_not_modified() {
                derives.push(quote! { ::core::default::Default });
            } else {
                let default_fields = attrs.entries();
                impls.push(quote! {
                    impl Default for #ident {
                        fn default() -> Self {
                            Self {
                                #(#default_fields),*
                            }
                        }
                    }
                });
            }
        } else {
            derives.push(quote! { ::core::default::Default });
        }
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
    #[cfg(feature = "serde")]
    if let Some(attrs) = &fields_attr {
        impls.push(attrs.serde_default_fns());
    }

    #[cfg(feature = "bytemuck")]
    if let Some(args) = attr.remove("pod") {
        derives.push(quote! { ::bytemuck::Pod });
        if !repr_c && !repr_transparent {
            reprs.push(quote! { C });
        }
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

    if let Some(mut args) = attr.remove("display") {
        if args.is_empty() {
            panic!("#[data(display)] requires arguments");
        }
        if let Some(args) = args.remove("debug") {
            impls.push(quote! {
                impl ::core::fmt::Display for #ident {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        write!(f, "{:?}", self)
                    }
                }
            });
            if !args.is_empty() {
                panic!("#[data(display(debug))] does not accept any arguments");
            }
        }
        if let Some(args) = args.remove("comma") {
            let Some(fields) = fields_list(input.clone()) else {
                panic!("#[data(display(comma))] can only be applied to structs");
            };
            impls.push(quote! {
                impl ::core::fmt::Display for #ident {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        let mut first = true;
                        #({
                            if !first {
                                write!(f, ",")?;
                                first = false;
                            }
                            write!(f, "{}", self.#fields)?;
                        })*
                        Ok(())
                    }
                }
            });
            if !args.is_empty() {
                panic!("#[data(display(comma))] does not accept any arguments");
            }
        }
        if let Some(args) = args.remove("semicolon") {
            let Some(fields) = fields_list(input.clone()) else {
                panic!("#[data(display(semicolon))] can only be applied to structs");
            };
            impls.push(quote! {
                impl ::core::fmt::Display for #ident {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        let mut first = true;
                        #({
                            if !first {
                                write!(f, ";")?;
                                first = false;
                            }
                            write!(f, "{}", self.#fields)?;
                        })*
                        Ok(())
                    }
                }
            });
            if !args.is_empty() {
                panic!("#[data(display(semicolon))] does not accept any arguments");
            }
        }
        if let Some(args) = args.remove("space") {
            let Some(fields) = fields_list(input.clone()) else {
                panic!("#[data(display(space))] can only be applied to structs");
            };
            impls.push(quote! {
                impl ::core::fmt::Display for #ident {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        let mut first = true;
                        #({
                            if !first {
                                write!(f, " ")?;
                                first = false;
                            }
                            write!(f, "{}", self.#fields)?;
                        })*
                        Ok(())
                    }
                }
            });
            if !args.is_empty() {
                panic!("#[data(display(space))] does not accept any arguments");
            }
        }
        if !args.is_empty() {
            panic!("Unsupported attribute for #[data(display)]: {args}");
        }
    }

    if let Some(mut args) = attr.remove("new") {
        if args.is_empty() {
            if let Some(attrs) = &fields_attr {
                let mut field_names = Vec::new();
                let mut field_types = Vec::new();
                let mut default_entries = Vec::new();
                for field in attrs.fields.iter() {
                    if let Some(new_value) = &field.new_value {
                        let name = &field.name;
                        match new_value {
                            Some(expr) => default_entries.push(quote! {
                                #name: #expr
                            }),
                            None => default_entries.push(quote! {
                                #name: ::core::default::Default::default()
                            }),
                        }
                    } else {
                        field_names.push(&field.name);
                        field_types.push(&field.ty);
                    }
                }
                impls.push(quote! {
                    impl #ident {
                        pub fn new(#(#field_names: #field_types),*) -> Self {
                            Self {
                                #(#field_names),*,
                                #(#default_entries),*
                            }
                        }
                    }
                });
            } else {
                panic!("#[data(new)] can only be applied to structs with named fields");
            }
        }
        if let Some(args) = args.remove("default") {
            if !enabled_attr.default {
                derives.push(quote! { ::core::default::Default });
            }
            impls.push(quote! {
                impl #ident {
                    pub fn new() -> Self {
                        Self::default()
                    }
                }
            });
            if !args.is_empty() {
                panic!("#[data(new(default))] does not accept any arguments");
            }
        }
        if !args.is_empty() {
            panic!("Unsupported attribute for #[data(new)]: {args}");
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

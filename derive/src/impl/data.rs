use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::DeriveInput;
use syn::spanned::Spanned;

use crate::util::data_helpers::{
    append_accessor_impls, append_builder_impl, append_deref_impls, append_validate_impl, error,
};
use crate::util::fields_attr::{Enabled as FieldsAttrEnabledFeatures, FieldsAttr};
use crate::util::parse_attr_tree::AttrArgs;

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

fn parse_all_attr_args(attr: TokenStream, mut input: DeriveInput) -> Result<AttrArgs, TokenStream> {
    let mut attrs = Vec::new();
    for attr in std::mem::take(&mut input.attrs) {
        if attr.path().is_ident("data") {
            let syn::Meta::List(attr) = attr.meta else {
                return error(attr.span(), "#[data(...)] attribute must be in list form");
            };
            attrs.push(attr.tokens.into());
        } else {
            input.attrs.push(attr);
        }
    }
    let mut args = parse_macro_input!(attr as AttrArgs)?;
    for attr in attrs {
        args.combine(parse_macro_input!(attr as AttrArgs)?);
    }
    Ok(args)
}

fn find_deref_span(input: &DeriveInput) -> Option<Span> {
    let syn::Data::Struct(data) = &input.data else {
        return None;
    };
    let fields = match &data.fields {
        syn::Fields::Named(fields) => &fields.named,
        syn::Fields::Unnamed(fields) => &fields.unnamed,
        syn::Fields::Unit => return None,
    };
    for field in fields {
        for attr in &field.attrs {
            if attr.path().is_ident("deref") {
                return Some(attr.span());
            }
        }
    }
    None
}

pub fn main(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut input = parse_macro_input!(item as DeriveInput)?;
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut attr = parse_all_attr_args(attr, input.clone())?;

    let mut reprs = Vec::new();
    let mut derives = Vec::new();
    let mut macros = Vec::new();
    let mut impls = Vec::new();
    let mut rkyv_derives = Vec::new();

    let no_eq = if let Some(args) = attr.remove("no-eq") {
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(no-eq)] does not accept any arguments",
            );
        }
        true
    } else {
        false
    };
    let no_ord = if let Some(args) = attr.remove("no-ord") {
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(no-ord)] does not accept any arguments",
            );
        }
        true
    } else if no_eq {
        true
    } else {
        false
    };

    derives.extend_from_slice(&[
        quote! { ::core::fmt::Debug },
        quote! { ::core::clone::Clone },
    ]);
    if !no_eq {
        derives.extend_from_slice(&[
            quote! { ::core::cmp::PartialEq },
            quote! { ::core::cmp::Eq },
        ]);
    }
    if !no_ord {
        derives.extend_from_slice(&[
            quote! { ::core::cmp::PartialOrd },
            quote! { ::core::cmp::Ord },
        ]);
    }
    derives.push(quote! { ::core::hash::Hash });

    rkyv_derives.extend_from_slice(&[quote! { ::core::fmt::Debug }]);
    if !no_eq {
        rkyv_derives.extend_from_slice(&[
            quote! { ::core::cmp::PartialEq },
            quote! { ::core::cmp::Eq },
        ]);
    }
    if !no_ord {
        rkyv_derives.extend_from_slice(&[
            quote! { ::core::cmp::PartialOrd },
            quote! { ::core::cmp::Ord },
        ]);
    }
    rkyv_derives.push(quote! { ::core::hash::Hash });

    macro_rules! repr_with_no_args {
        ($name:expr, $quote:expr) => {
            if let Some(args) = attr.remove($name) {
                reprs.push($quote);
                if !args.is_empty() {
                    return error(
                        Span::call_site(),
                        format!("#[data({})] does not accept any arguments", $name),
                    );
                }
            }
        };
    }

    #[allow(unused_variables)]
    let repr_c = attr.get("raw").is_some();
    #[allow(unused_variables)]
    let repr_transparent = attr.get("transparent").is_some();
    #[allow(unused_variables)]
    let derive_copy = attr.get("copy").is_some();

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
        deref: true,
        accessors: true,
        builder: true,
        validate: true,
        add_comment_on_changed: true,
    };
    let fields_attr = match &mut input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => Some(
            FieldsAttr::parse(ident, &input.generics, &mut fields.named, enabled_attr)
                .map_err(|e| TokenStream::from(e.to_compile_error()))?,
        ),
        _ => None,
    };
    if fields_attr.is_none() {
        if let Some(span) = find_deref_span(&input) {
            return error(
                span,
                "#[deref] can only be applied to structs with named fields",
            );
        }
    }
    if attr.get("builder").is_none() {
        if let Some(attrs) = &fields_attr {
            if attrs.fields.iter().any(|f| f.builder_default) {
                return error(
                    Span::call_site(),
                    "#[builder(default)] requires #[data(builder)]",
                );
            }
        }
    }

    if let Some(args) = attr.remove("default") {
        if let Some(attrs) = &fields_attr {
            if attrs.default_not_modified() {
                derives.push(quote! { ::core::default::Default });
            } else {
                let default_fields = attrs.entries();
                impls.push(quote! {
                    impl #impl_generics Default for #ident #ty_generics #where_clause {
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
            return error(
                Span::call_site(),
                "#[data(default)] does not accept any arguments",
            );
        }
    }

    if let Some(args) = attr.remove("copy") {
        derives.push(quote! { ::core::marker::Copy });
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(copy)] does not accept any arguments",
            );
        }
    }

    if let Some(args) = attr.remove("to-*") {
        macro_rules! handle_wildcard {
            ($wildcard:expr, $name:expr) => {
                if attr
                    .insert($name.to_string(), AttrArgs::default())
                    .is_some()
                {
                    return error(
                        Span::call_site(),
                        format!(
                            "#[data({})] is duplicate when using #[data({})]",
                            $name, $wildcard
                        ),
                    );
                }
            };
        }
        handle_wildcard!("to-*", "to-prev");
        handle_wildcard!("to-*", "to-next");
        #[cfg(feature = "rand")]
        handle_wildcard!("to-*", "to-random");
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(to-prev)] does not accept any arguments",
            );
        }
    }

    data_to_xxx(&mut derives, &mut attr)?;

    #[cfg(not(feature = "rkyv"))]
    let mut rkyv: Option<AttrArgs> = None;
    #[cfg(feature = "rkyv")]
    let mut rkyv = attr.remove("rkyv");

    #[cfg(feature = "rkyv")]
    if rkyv.is_some() {
        derives.push(quote! { ::data_classes::deps::rkyv::Archive });
        derives.push(quote! { ::data_classes::deps::rkyv::Serialize });
        derives.push(quote! { ::data_classes::deps::rkyv::Deserialize });
    }

    #[cfg(feature = "serde")]
    data_serde(&mut derives, &mut macros, repr_transparent, &mut attr)?;
    #[cfg(feature = "serde")]
    if let Some(attrs) = &fields_attr {
        impls.push(attrs.serde_default_fns());
    }

    append_deref_impls(
        &fields_attr,
        &impl_generics,
        &ty_generics,
        where_clause,
        &mut impls,
    );
    append_accessor_impls(
        &fields_attr,
        &impl_generics,
        &ty_generics,
        where_clause,
        &mut impls,
    );
    append_validate_impl(
        &mut attr,
        &fields_attr,
        &impl_generics,
        &ty_generics,
        where_clause,
        &mut impls,
    )?;
    append_builder_impl(
        &mut attr,
        &fields_attr,
        ident,
        &input.generics,
        &impl_generics,
        &ty_generics,
        where_clause,
        &mut impls,
    )?;

    #[cfg(feature = "bytemuck")]
    if let Some(args) = attr.remove("pod") {
        derives.push(quote! { ::data_classes::deps::bytemuck::Pod });
        // Bytemuck requires repr(C) or repr(transparent) for Pod types
        // and we add repr(C) automatically if neither is specified.
        if !repr_c && !repr_transparent {
            reprs.push(quote! { C });
        }
        // Bytemuck also requires Zeroable for Pod types
        let _ = attr.remove("zeroable");
        derives.push(quote! { ::data_classes::deps::bytemuck::Zeroable });
        // And Copy should also be derived
        if !derive_copy {
            derives.push(quote! { ::core::marker::Copy });
        }
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(pod)] does not accept any arguments",
            );
        }
    }

    #[cfg(feature = "bytemuck")]
    if let Some(args) = attr.remove("zeroable") {
        derives.push(quote! { ::data_classes::deps::bytemuck::Zeroable });
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(zeroable)] does not accept any arguments",
            );
        }
    }

    if let Some(mut args) = attr.remove("display") {
        if args.is_empty() {
            return error(Span::call_site(), "#[data(display)] requires arguments");
        }
        if let Some(args) = args.remove("debug") {
            impls.push(quote! {
                impl #impl_generics ::core::fmt::Display for #ident #ty_generics #where_clause {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        write!(f, "{:?}", self)
                    }
                }
            });
            if !args.is_empty() {
                return error(
                    Span::call_site(),
                    "#[data(display(debug))] does not accept any arguments",
                );
            }
        }
        if let Some(args) = args.remove("comma") {
            let Some(fields) = fields_list(input.clone()) else {
                return error(
                    Span::call_site(),
                    "#[data(display(comma))] can only be applied to structs",
                );
            };
            impls.push(quote! {
                impl #impl_generics ::core::fmt::Display for #ident #ty_generics #where_clause {
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
                return error(
                    Span::call_site(),
                    "#[data(display(comma))] does not accept any arguments",
                );
            }
        }
        if let Some(args) = args.remove("semicolon") {
            let Some(fields) = fields_list(input.clone()) else {
                return error(
                    Span::call_site(),
                    "#[data(display(semicolon))] can only be applied to structs",
                );
            };
            impls.push(quote! {
                impl #impl_generics ::core::fmt::Display for #ident #ty_generics #where_clause {
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
                return error(
                    Span::call_site(),
                    "#[data(display(semicolon))] does not accept any arguments",
                );
            }
        }
        if let Some(args) = args.remove("space") {
            let Some(fields) = fields_list(input.clone()) else {
                return error(
                    Span::call_site(),
                    "#[data(display(space))] can only be applied to structs",
                );
            };
            impls.push(quote! {
                impl #impl_generics ::core::fmt::Display for #ident #ty_generics #where_clause {
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
                return error(
                    Span::call_site(),
                    "#[data(display(space))] does not accept any arguments",
                );
            }
        }
        if !args.is_empty() {
            return error(
                Span::call_site(),
                format!("Unsupported attribute for #[data(display)]: {args}"),
            );
        }
    }

    if let Some(mut args) = attr.remove("new") {
        let is_const_or_not = args.remove("const");
        if let Some(ref is_const_or_not) = is_const_or_not
            && !is_const_or_not.is_empty()
        {
            return error(
                Span::call_site(),
                "#[data(new(const))] does not accept any arguments",
            );
        }
        let is_const_or_not = if is_const_or_not.is_some() {
            quote! { const }
        } else {
            quote! {}
        };
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
                    impl #impl_generics #ident #ty_generics #where_clause {
                        pub #is_const_or_not fn new(#(#field_names: #field_types),*) -> Self {
                            Self {
                                #(#field_names),*,
                                #(#default_entries),*
                            }
                        }
                    }
                });
            } else {
                return error(
                    Span::call_site(),
                    "#[data(new)] can only be applied to structs with named fields",
                );
            }
        }
        if let Some(args) = args.remove("default") {
            if !enabled_attr.default {
                derives.push(quote! { ::core::default::Default });
            }
            impls.push(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub #is_const_or_not fn new() -> Self {
                        Self::default()
                    }
                }
            });
            if !args.is_empty() {
                return error(
                    Span::call_site(),
                    "#[data(new(default))] does not accept any arguments",
                );
            }
        }
        if !args.is_empty() {
            return error(
                Span::call_site(),
                format!("Unsupported attribute for #[data(new)]: {args}"),
            );
        }
    }

    if !attr.is_empty() {
        return error(
            Span::call_site(),
            format!("Unsupported attribute for #[data]: {attr}"),
        );
    }

    if rkyv.is_some() {
        macros.push(quote! { #[rkyv(derive(#(#rkyv_derives),*))] });
    }

    if let Some(rkyv) = &mut rkyv {
        if let Some(args) = rkyv.remove("no-cmp") {
            if !args.is_empty() {
                return error(
                    Span::call_site(),
                    "#[data(rkyv(no-cmp))] does not accept any arguments",
                );
            }
        } else {
            macros.push(quote! { #[rkyv(compare(PartialEq, PartialOrd))] });
        }
    }

    if rkyv.is_some()
        && let Some(args) = rkyv.as_mut().unwrap().remove("omit-bounds")
    {
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(rkyv(omit-bounds))] does not accept any arguments",
            );
        }
        macros.push(quote! {
            #[rkyv(serialize_bounds(__S: ::rkyv::ser::Writer + ::rkyv::ser::Allocator, __S::Error: ::rkyv::rancor::Source))]
            #[rkyv(deserialize_bounds(__D::Error: ::rkyv::rancor::Source))]
            #[rkyv(bytecheck(bounds(__C: ::rkyv::validation::ArchiveContext)))]
        });
    }

    if let Some(rkyv) = rkyv
        && !rkyv.is_empty()
    {
        return error(
            Span::call_site(),
            format!("Unsupported attribute for #[data(rkyv)]: {rkyv}"),
        );
    }

    let expanded = quote! {
        #[repr(#(#reprs),*)]
        #[derive(#(#derives),*)]
        #(#macros)*
        #input
        #(#impls)*
    };

    Ok(TokenStream::from(expanded))
}

fn data_serde(
    derives: &mut Vec<TokenStream2>,
    macros: &mut Vec<TokenStream2>,
    repr_transparent: bool,
    attr: &mut AttrArgs,
) -> Result<(), TokenStream> {
    if let Some(mut args) = attr.remove("serde") {
        if args.is_empty() {
            derives.push(quote! { ::data_classes::deps::serde::Serialize });
            derives.push(quote! { ::data_classes::deps::serde::Deserialize });
        }
        if let Some(args) = args.remove("s") {
            derives.push(quote! { ::data_classes::deps::serde::Serialize });
            if !args.is_empty() {
                return error(
                    Span::call_site(),
                    "#[data(serde(s))] does not accept any arguments",
                );
            }
        }
        if let Some(args) = args.remove("d") {
            derives.push(quote! { ::data_classes::deps::serde::Deserialize });
            if !args.is_empty() {
                return error(
                    Span::call_site(),
                    "#[data(serde(d))] does not accept any arguments",
                );
            }
        }
        if !args.is_empty() {
            return error(
                Span::call_site(),
                format!("#[data(serde)] has unsupported arguments: {args}"),
            );
        }
        if repr_transparent {
            macros.push(quote! { #[serde(transparent)] });
        }
    }
    Ok(())
}

fn data_to_xxx(derives: &mut Vec<TokenStream2>, attr: &mut AttrArgs) -> Result<(), TokenStream> {
    if let Some(args) = attr.remove("to-*") {
        macro_rules! handle_wildcard {
            ($wildcard:expr, $name:expr) => {
                if attr
                    .insert($name.to_string(), AttrArgs::default())
                    .is_some()
                {
                    return error(
                        Span::call_site(),
                        format!(
                            "#[data({})] is duplicate when using #[data({})]",
                            $name, $wildcard
                        ),
                    );
                }
            };
        }
        handle_wildcard!("to-*", "to-prev");
        handle_wildcard!("to-*", "to-next");
        #[cfg(feature = "rand")]
        handle_wildcard!("to-*", "to-random");
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(to-prev)] does not accept any arguments",
            );
        }
    }

    if let Some(args) = attr.remove("to-prev") {
        derives.push(quote! { ::data_classes::derive::ToPrev });
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(to-prev)] does not accept any arguments",
            );
        }
    }

    if let Some(args) = attr.remove("to-next") {
        derives.push(quote! { ::data_classes::derive::ToNext });
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(to-next)] does not accept any arguments",
            );
        }
    }

    #[cfg(feature = "rand")]
    if let Some(args) = attr.remove("to-random") {
        derives.push(quote! { ::data_classes::derive::ToRandom });
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(to-random)] does not accept any arguments",
            );
        }
    }
    Ok(())
}

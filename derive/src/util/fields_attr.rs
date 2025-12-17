use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Meta, MetaList, MetaNameValue, Token};

pub struct EnabledAttrs {
    pub default: bool,
    pub new: bool,
}

#[derive(Clone)]
pub struct FieldAttr {
    pub name: syn::Ident,
    pub ty: syn::Type,
    pub default: Option<syn::Expr>,
    pub new_value: Option<Option<syn::Expr>>,
}

impl FieldAttr {
    fn new(name: syn::Ident, ty: syn::Type) -> Self {
        FieldAttr {
            name,
            ty,
            default: None,
            new_value: None,
        }
    }

    fn entry(&self) -> TokenStream2 {
        let name = &self.name;
        if let Some(value) = &self.default {
            quote! { #name: #value }
        } else {
            quote! { #name: ::core::default::Default::default() }
        }
    }
}

pub struct FieldsAttr {
    pub fields: Vec<FieldAttr>,
    pub fields_map: HashMap<String, FieldAttr>,
}

impl FieldsAttr {
    pub fn parse(fields: &mut Punctuated<syn::Field, Token![,]>, enabled: &EnabledAttrs) -> Self {
        let mut default_fields = Vec::new();

        for field in fields {
            let name: &syn::Ident = field.ident.as_ref().unwrap();
            let mut output = FieldAttr::new(name.clone(), field.ty.clone());
            field.attrs.retain(|attr| {
                if enabled.default && attr.path().is_ident("default") {
                    let Meta::NameValue(ref val) = attr.meta else {
                        panic!("The #[default] attribute must be in the form #[default = ...] for field {name}");
                    };
                    if output.default.replace(val.value.clone()).is_some() {
                        panic!("The #[default = ...] attribute for field {name} can only be specified once");
                    }
                    return false;
                }
                true
            });
            field.attrs.retain(|attr| {
                if enabled.new && attr.path().is_ident("new") {
                    // match attr.meta {
                    //     Meta::List(MetaList { tokens, .. }) => {}
                    //     Meta::NameValue(MetaNameValue { value, .. }) => {}
                    // }
                    let Meta::NameValue(ref val) = attr.meta else {
                        panic!(
                            "The #[new] attribute must be in the form #[new = ...] for field {name}"
                        );
                    };
                    if let Expr::Infer(_) = val.value {
                        if output.new_value.replace(None).is_some() {
                            panic!(
                                "The #[new = _] attribute for field {name} can only be specified once"
                            );
                        }
                    } else if output.new_value.replace(Some(val.value.clone())).is_some() {
                        panic!(
                            "The #[new = ...] attribute for field {name} can only be specified once"
                        );
                    }
                    return false;
                }
                true
            });
            default_fields.push(output);
        }

        let mut fields_map = HashMap::new();
        for field in default_fields.iter().cloned() {
            fields_map.insert(field.name.to_string(), field);
        }

        FieldsAttr {
            fields: default_fields,
            fields_map,
        }
    }

    pub fn default_not_modified(&self) -> bool {
        self.fields.iter().all(|f| f.default.is_none())
    }

    pub fn entries(&self) -> Vec<TokenStream2> {
        self.fields.iter().map(|f| f.entry()).collect()
    }
}

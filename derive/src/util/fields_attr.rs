use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens as _, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Meta, Token};

pub struct Enabled {
    /// Whether to parse the #[default] attribute.
    pub default: bool,
    /// Whether to parse the #[new] attribute.
    pub new: bool,
    /// Add comments to fields whose default values ​​have been changed.<br />
    /// Add comments to fields whose new values ​​have been specified.<br />
    pub add_comment_on_changed: bool,
}

#[derive(Clone)]
pub struct FieldAttr {
    pub name: syn::Ident,
    pub ty: syn::Type,
    pub default: Option<syn::Expr>,
    pub new_value: Option<Option<syn::Expr>>,
    pub serde_default: Option<TokenStream2>,
}

impl FieldAttr {
    fn new(name: syn::Ident, ty: syn::Type) -> Self {
        FieldAttr {
            name,
            ty,
            default: None,
            new_value: None,
            serde_default: None,
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
    pub ident: Ident,
    pub fields: Vec<FieldAttr>,
    pub fields_map: HashMap<String, FieldAttr>,
}

impl FieldsAttr {
    pub fn parse(
        ident: &Ident,
        fields: &mut Punctuated<syn::Field, Token![,]>,
        enabled: &Enabled,
    ) -> Self {
        let mut default_fields = Vec::new();

        for field in fields {
            let name: &syn::Ident = field.ident.as_ref().unwrap();
            let mut doc = Vec::new();
            for attr in std::mem::take(&mut field.attrs) {
                if attr.path().is_ident("doc") {
                    doc.push(attr);
                    continue;
                }
                field.attrs.push(attr);
            }
            let mut comment = Vec::new();
            let mut output = FieldAttr::new(name.clone(), field.ty.clone());
            for attr in std::mem::take(&mut field.attrs) {
                if !enabled.default || !attr.path().is_ident("default") {
                    field.attrs.push(attr);
                    continue;
                }
                let Meta::NameValue(ref val) = attr.meta else {
                    panic!(
                        "The #[default] attribute must be in the form #[default = ...] for field {name}"
                    );
                };
                if output.default.replace(val.value.clone()).is_some() {
                    panic!(
                        "The #[default = ...] attribute for field {name} can only be specified once"
                    );
                }
                let val = &val.value;
                comment.push(format!("default: `` {} ``", quote! { #val }));
            }
            for attr in std::mem::take(&mut field.attrs) {
                if !enabled.new || !attr.path().is_ident("new") {
                    field.attrs.push(attr);
                    continue;
                }
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
                    comment.push("new: *use default*".to_string());
                } else {
                    if output.new_value.replace(Some(val.value.clone())).is_some() {
                        panic!(
                            "The #[new = ...] attribute for field {name} can only be specified once"
                        );
                    }
                    let val = &val.value;
                    comment.push(format!("new: `` {} ``", quote! { #val }));
                }
            }
            if output.default.is_some() {
                let mut default_fn = None;
                for attr in std::mem::take(&mut field.attrs) {
                    if !attr.path().is_ident("serde") {
                        field.attrs.push(attr);
                        continue;
                    }
                    let Meta::List(_) = attr.meta else {
                        field.attrs.push(attr);
                        continue;
                    };
                    let mut metas = Vec::new();
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") && meta.input.is_empty() {
                            let fn_id = format!("{ident}::__data_classes__serde_default__{name}");
                            let expr = syn::LitStr::new(&fn_id, meta.path.span());
                            metas.push(quote! { default = #expr });
                            let fn_name = format!("__data_classes__serde_default__{name}");
                            let fn_name = syn::Ident::new(&fn_name, proc_macro2::Span::call_site());
                            default_fn = Some(fn_name);
                        } else {
                            let mut ts = meta.path.get_ident().unwrap().into_token_stream();
                            ts.extend(meta.input.parse::<TokenStream2>().unwrap());
                            metas.push(ts);
                        }
                        Ok(())
                    });
                    let mut attr = attr;
                    if let Meta::List(list) = &mut attr.meta {
                        list.tokens = quote! { #( #metas ),* };
                    };
                    field.attrs.push(attr);
                }
                if let Some(fn_name) = default_fn {
                    let ty = &output.ty;
                    let default = output.default.as_ref().unwrap();
                    output.serde_default = Some(quote! {
                        fn #fn_name() -> #ty {
                            #default
                        }
                    });
                }
            }
            default_fields.push(output);
            if enabled.add_comment_on_changed && !comment.is_empty() {
                if !doc.is_empty() {
                    doc.push(syn::parse_quote! { #[doc = ""] });
                    doc.push(syn::parse_quote! { #[doc = "---"] });
                    doc.push(syn::parse_quote! { #[doc = ""] });
                }
                for c in comment {
                    doc.push(syn::parse_quote! { #[doc = #c] });
                }
            }
            field.attrs.extend(doc);
        }

        let mut fields_map = HashMap::new();
        for field in default_fields.iter().cloned() {
            fields_map.insert(field.name.to_string(), field);
        }

        FieldsAttr {
            ident: ident.clone(),
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

    pub fn serde_default_fns(&self) -> TokenStream2 {
        let ident = &self.ident;
        let fns = self
            .fields
            .iter()
            .filter_map(|f| f.serde_default.as_ref())
            .collect::<Vec<_>>();
        if fns.is_empty() {
            quote! {}
        } else {
            quote! {
                impl #ident {
                    #(#fns)*
                }
            }
        }
    }
}

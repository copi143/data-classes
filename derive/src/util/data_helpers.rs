use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{ImplGenerics, TypeGenerics, WhereClause};

use crate::util::{fields_attr::FieldsAttr, parse_attr_tree::AttrArgs};

pub(crate) fn error<T>(span: Span, msg: impl std::fmt::Display) -> Result<T, TokenStream> {
    Err(TokenStream::from(
        syn::Error::new(span, msg.to_string()).to_compile_error(),
    ))
}

pub(crate) fn append_deref_impls(
    fields_attr: &Option<FieldsAttr>,
    impl_generics: &ImplGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    impls: &mut Vec<TokenStream2>,
) {
    let Some(attrs) = fields_attr else {
        return;
    };
    let ident = &attrs.ident;
    if let Some((name, ty, is_mut)) = attrs.deref_target() {
        impls.push(quote! {
            impl #impl_generics ::core::ops::Deref for #ident #ty_generics #where_clause {
                type Target = #ty;
                fn deref(&self) -> &Self::Target {
                    &self.#name
                }
            }
        });
        if is_mut {
            impls.push(quote! {
                impl #impl_generics ::core::ops::DerefMut for #ident #ty_generics #where_clause {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.#name
                    }
                }
            });
        }
    }
}

pub(crate) fn append_accessor_impls(
    fields_attr: &Option<FieldsAttr>,
    impl_generics: &ImplGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    impls: &mut Vec<TokenStream2>,
) {
    let Some(attrs) = fields_attr else {
        return;
    };
    let ident = &attrs.ident;
    for field in attrs.fields.iter() {
        let name = &field.name;
        let ty = &field.ty;
        let get_name = syn::Ident::new(&format!("get_{}", name), Span::call_site());
        let get_mut_name = syn::Ident::new(&format!("get_{}_mut", name), Span::call_site());
        let set_name = syn::Ident::new(&format!("set_{}", name), Span::call_site());
        let with_name = syn::Ident::new(&format!("with_{}", name), Span::call_site());
        if field.get {
            impls.push(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub fn #get_name(&self) -> &#ty {
                        &self.#name
                    }
                }
            });
        }
        if field.get_mut {
            impls.push(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub fn #get_mut_name(&mut self) -> &mut #ty {
                        &mut self.#name
                    }
                }
            });
        }
        if field.set {
            let check = field.where_expr.as_ref().map(|expr| {
                quote! {
                    let #name = &value;
                    if !(#expr) {
                        panic!("check failed for field {}", stringify!(#name));
                    }
                }
            });
            impls.push(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub fn #set_name(&mut self, value: #ty) {
                        #check
                        self.#name = value;
                    }
                }
            });
        }
        if field.with {
            let check = field.where_expr.as_ref().map(|expr| {
                quote! {
                    let #name = &value;
                    if !(#expr) {
                        panic!("check failed for field {}", stringify!(#name));
                    }
                }
            });
            impls.push(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub fn #with_name(mut self, value: #ty) -> Self {
                        #check
                        self.#name = value;
                        self
                    }
                }
            });
        }
    }
}

pub(crate) fn append_validate_impl(
    attr: &mut AttrArgs,
    fields_attr: &Option<FieldsAttr>,
    impl_generics: &ImplGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    impls: &mut Vec<TokenStream2>,
) -> Result<(), TokenStream> {
    if let Some(args) = attr.remove("validate") {
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(validate)] does not accept any arguments",
            );
        }
        let Some(attrs) = fields_attr else {
            return error(
                Span::call_site(),
                "#[data(validate)] can only be applied to structs with named fields",
            );
        };
        let checks = attrs.validate_entries();
        let binds = checks
            .iter()
            .map(|(name, _)| quote! { let #name = &self.#name; });
        let exprs = checks.iter().map(|(_, expr)| quote! { (#expr) });
        let body = if checks.is_empty() {
            quote! { true }
        } else {
            quote! { true #(&& #exprs)* }
        };
        let ident = &attrs.ident;
        impls.push(quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                pub fn validate(&self) -> bool {
                    #(#binds)*
                    #body
                }
            }
        });
    }
    Ok(())
}

pub(crate) fn append_builder_impl(
    attr: &mut AttrArgs,
    fields_attr: &Option<FieldsAttr>,
    ident: &syn::Ident,
    generics: &syn::Generics,
    impl_generics: &ImplGenerics<'_>,
    ty_generics: &TypeGenerics<'_>,
    where_clause: Option<&WhereClause>,
    impls: &mut Vec<TokenStream2>,
) -> Result<(), TokenStream> {
    if let Some(args) = attr.remove("builder") {
        if !args.is_empty() {
            return error(
                Span::call_site(),
                "#[data(builder)] does not accept any arguments",
            );
        }
        let Some(attrs) = fields_attr else {
            return error(
                Span::call_site(),
                "#[data(builder)] can only be applied to structs with named fields",
            );
        };
        let builder_ident = syn::Ident::new(&format!("{}Builder", ident), Span::call_site());
        let builder_generics = generics;
        let builder_fields = attrs.fields.iter().map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            if f.builder_default {
                quote! { #name: #ty }
            } else {
                quote! { #name: ::core::option::Option<#ty> }
            }
        });
        let builder_inits = attrs.fields.iter().map(|f| {
            let name = &f.name;
            if f.builder_default {
                quote! { #name: ::core::default::Default::default() }
            } else {
                quote! { #name: ::core::option::Option::None }
            }
        });
        let with_fns = attrs.fields.iter().map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            let with_name = syn::Ident::new(&format!("with_{}", name), Span::call_site());
            let assign = if f.builder_default {
                quote! { self.#name = value; }
            } else {
                quote! { self.#name = ::core::option::Option::Some(value); }
            };
            quote! {
                pub fn #with_name(mut self, value: #ty) -> Self {
                    #assign
                    self
                }
            }
        });
        let build_fields = attrs.fields.iter().map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            let check = f.where_expr.as_ref().map(|expr| {
                quote! {
                    let #name = &value;
                    if !(#expr) {
                        panic!("check failed for field {}", stringify!(#name));
                    }
                }
            });
            if f.builder_default {
                quote! {
                    let value: #ty = self.#name;
                    #check
                    let #name = value;
                }
            } else {
                quote! {
                    let value: #ty = match self.#name {
                        ::core::option::Option::Some(value) => value,
                        ::core::option::Option::None => {
                            panic!("missing field {}", stringify!(#name));
                        }
                    };
                    #check
                    let #name = value;
                }
            }
        });
        let build_init = attrs.fields.iter().map(|f| {
            let name = &f.name;
            quote! { #name }
        });
        impls.push(quote! {
            pub struct #builder_ident #builder_generics {
                #(#builder_fields),*
            }

            impl #impl_generics #builder_ident #ty_generics #where_clause {
                #(#with_fns)*

                pub fn build(self) -> #ident #ty_generics {
                    #(#build_fields)*
                    #ident { #(#build_init),* }
                }
            }

            impl #impl_generics #ident #ty_generics #where_clause {
                pub fn builder() -> #builder_ident #ty_generics {
                    #builder_ident {
                        #(#builder_inits),*
                    }
                }
            }
        });
    }
    Ok(())
}

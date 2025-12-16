use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Data, DataStruct, DeriveInput, Fields, Meta, Token, parse_macro_input};

pub fn collect_default_fields(fields: &mut Punctuated<syn::Field, Token![,]>) -> Vec<TokenStream2> {
    let mut default_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let mut default_val = None;
        field.attrs.retain(|attr| {
            if let Meta::NameValue(ref val) = attr.meta
                && val.path.is_ident("default")
            {
                let _ = default_val.replace(val.value.clone()).is_none_or(|_| {
                    panic!("The #[default = ...] attribute for field {field_name} can only be specified once")
                });
                false
            } else {
                true
            }
        });
        let def = if let Some(val) = default_val {
            quote! { #field_name: #val }
        } else {
            quote! { #field_name: Default::default() }
        };
        default_fields.push(def);
    }

    default_fields
}

pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.to_string().trim() != "" {
        panic!("#[default] does not accept any arguments");
    }

    let mut input = parse_macro_input!(item as DeriveInput);
    let ident = &input.ident;

    let default_fields = collect_default_fields(match &mut input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &mut fields.named,
        _ => panic!("#[default] can only be applied to structs with named fields"),
    });

    let expanded = quote! {
        #input

        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

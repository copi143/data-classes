use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

use crate::util::fields_attr::{EnabledAttrs, FieldsAttr};

pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.to_string().trim() != "" {
        panic!("#[default] does not accept any arguments");
    }

    let mut input = parse_macro_input!(item as DeriveInput);
    let ident = &input.ident;

    let enabled_attrs = &EnabledAttrs {
        default: true,
        new: false,
    };
    let fields_attr = FieldsAttr::parse(
        match &mut input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &mut fields.named,
            _ => panic!("#[default] can only be applied to structs with named fields"),
        },
        enabled_attrs,
    );

    let expanded = if fields_attr.default_not_modified() {
        quote! {
            #[derive(::core::default::Default)]
            #input
        }
    } else {
        let default_fields = fields_attr.entries();
        quote! {
            #input

            impl Default for #ident {
                fn default() -> Self {
                    Self {
                        #(#default_fields),*
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

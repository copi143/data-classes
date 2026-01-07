use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

use crate::util::fields_attr::{Enabled, FieldsAttr};

pub fn main(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    if attr.to_string().trim() != "" {
        return Err(TokenStream::from(
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[default] does not accept any arguments",
            )
            .to_compile_error(),
        ));
    }

    let mut input = parse_macro_input!(item as DeriveInput)?;
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let enabled_attrs = &Enabled {
        default: true,
        new: false,
        deref: false,
        accessors: false,
        builder: false,
        validate: false,
        add_comment_on_changed: true,
    };
    let fields_attr = FieldsAttr::parse(
        ident,
        &input.generics,
        match &mut input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &mut fields.named,
            _ => {
                return Err(TokenStream::from(
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "#[default] can only be applied to structs with named fields",
                    )
                    .to_compile_error(),
                ));
            }
        },
        enabled_attrs,
    )
    .map_err(|e| TokenStream::from(e.to_compile_error()))?;

    let expanded = if fields_attr.default_not_modified() {
        quote! {
            #[derive(::core::default::Default)]
            #input
        }
    } else {
        let default_fields = fields_attr.entries();
        quote! {
            #input

            impl #impl_generics Default for #ident #ty_generics #where_clause {
                fn default() -> Self {
                    Self {
                        #(#default_fields),*
                    }
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

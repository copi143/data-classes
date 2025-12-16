use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Token, Type, Visibility};

/// Function-like proc-macro to define a struct with per-field initializers.
///
/// Usage:
///
/// init_struct! {
///     pub struct Foo {
///         pub a: i32 = 3,
///         pub b: String = "hi".to_string(),
///         c: u8,
///     }
/// }
///
/// This expands to a normal `pub struct Foo { ... }` and
/// `impl Foo { pub fn new() -> Self { ... } }` plus `impl Default`.
struct FieldInit {
    vis: Visibility,
    name: Ident,
    ty: Type,
    init: Option<Expr>,
}

struct InitStruct {
    vis: Visibility,
    struct_token: Token![struct],
    name: Ident,
    brace_token: syn::token::Brace,
    fields: Punctuated<FieldInit, Token![,]>,
}

impl Parse for FieldInit {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        let init = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(FieldInit {
            vis,
            name,
            ty,
            init,
        })
    }
}

impl Parse for InitStruct {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let vis: Visibility = input.parse()?;
        let struct_token: Token![struct] = input.parse()?;
        let name: Ident = input.parse()?;
        let content;
        let brace_token = syn::braced!(content in input);
        let mut fields: Punctuated<FieldInit, Token![,]> = Punctuated::new();
        while !content.is_empty() {
            let f: FieldInit = content.parse()?;
            fields.push(f);
            if content.peek(Token![,]) {
                let _comma: Token![,] = content.parse()?;
            } else {
                break;
            }
        }
        Ok(InitStruct {
            vis,
            struct_token,
            name,
            brace_token,
            fields,
        })
    }
}

pub fn main(item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as InitStruct);
    let vis = parsed.vis;
    let name = parsed.name;

    // Build field declarations (without init expr) and initializer expressions
    let mut decls = Vec::new();
    let mut inits = Vec::new();

    for f in parsed.fields.iter() {
        let fvis = &f.vis;
        let fname = &f.name;
        let fty = &f.ty;
        decls.push(quote! { #fvis #fname: #fty, });
        if let Some(expr) = &f.init {
            inits.push(quote! { #fname: #expr, });
        } else {
            inits.push(quote! { #fname: ::std::default::Default::default(), });
        }
    }

    let expanded = quote! {
        #vis struct #name {
            #(#decls)*
        }

        impl #name {
            /// Construct a new instance using the field initializers.
            pub fn new() -> Self {
                Self {
                    #(#inits)*
                }
            }
        }

        impl ::std::default::Default for #name {
            fn default() -> Self { Self::new() }
        }
    };

    expanded.into()
}

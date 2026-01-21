use proc_macro::TokenStream;

macro_rules! parse_macro_input {
    ($ts:ident as $ty:ty) => {
        match syn::parse::<$ty>($ts) {
            Ok(data) => Ok(data),
            Err(err) => Err(TokenStream::from(err.to_compile_error())),
        }
    };
    ($ts:ident with $parser:path) => {
        match syn::parse::Parser::parse($parser, $ts) {
            Ok(data) => Ok(data),
            Err(err) => Err(TokenStream::from(err.to_compile_error())),
        }
    };
    ($ts:ident) => {
        parse_macro_input!($ts as _)
    };
}

mod r#impl {
    pub mod data;
    pub mod default;
    pub mod derive;
    pub mod init_struct;
    pub mod keyval;
}

mod util {
    pub mod data_helpers;
    pub mod fields_attr;
    pub mod parse_attr_tree;
}

#[proc_macro]
pub fn init_struct(item: TokenStream) -> TokenStream {
    r#impl::init_struct::main(item)
}

#[proc_macro_attribute]
pub fn data(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::data::main(attr, item).unwrap_or_else(|e| e)
}

#[proc_macro_attribute]
pub fn key(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::keyval::key(attr, item).unwrap_or_else(|e| e)
}

#[proc_macro_attribute]
pub fn val(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::keyval::val(attr, item).unwrap_or_else(|e| e)
}

#[proc_macro_attribute]
pub fn default(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::default::main(attr, item).unwrap_or_else(|e| e)
}

#[proc_macro_derive(ToPrev)]
pub fn derive_to_prev(item: TokenStream) -> TokenStream {
    r#impl::derive::to_prev(item).unwrap_or_else(|e| e)
}

#[proc_macro_derive(ToNext)]
pub fn derive_to_next(item: TokenStream) -> TokenStream {
    r#impl::derive::to_next(item).unwrap_or_else(|e| e)
}

#[cfg(feature = "rand")]
#[proc_macro_derive(ToRandom)]
pub fn derive_to_random(item: TokenStream) -> TokenStream {
    r#impl::derive::to_random(item).unwrap_or_else(|e| e)
}

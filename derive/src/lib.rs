use proc_macro::TokenStream;

mod r#impl {
    pub mod data;
    pub mod default;
    pub mod init_struct;
    pub mod keyval;
}

mod util {
    pub mod parse_attr_tree;
}

#[proc_macro]
pub fn init_struct(item: TokenStream) -> TokenStream {
    r#impl::init_struct::main(item)
}

#[proc_macro_attribute]
pub fn data(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::data::main(attr, item)
}

#[proc_macro_attribute]
pub fn key(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::keyval::key(attr, item)
}

#[proc_macro_attribute]
pub fn val(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::keyval::val(attr, item)
}

#[proc_macro_attribute]
pub fn default(attr: TokenStream, item: TokenStream) -> TokenStream {
    r#impl::default::main(attr, item)
}

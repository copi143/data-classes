use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, parse_macro_input};

/// 为 enum 自动实现 prev/next 方法
pub fn to_prev(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let variants: Vec<&Ident> = input.variants.iter().map(|v| &v.ident).collect();
    let len = variants.len();
    if len == 0 {
        return TokenStream::new();
    }
    let mut match_arms = Vec::new();
    for (i, ident) in variants.iter().enumerate() {
        let prev = if i == 0 {
            variants[len - 1]
        } else {
            variants[i - 1]
        };
        match_arms.push(quote! { #enum_name::#ident => #enum_name::#prev });
    }
    let expanded = quote! {
        impl ::data_classes::ToPrev for #enum_name {
            fn get_prev(&self) -> Self {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    };
    TokenStream::from(expanded)
}

pub fn to_next(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let variants: Vec<&Ident> = input.variants.iter().map(|v| &v.ident).collect();
    let len = variants.len();
    if len == 0 {
        return TokenStream::new();
    }
    let mut match_arms = Vec::new();
    for (i, ident) in variants.iter().enumerate() {
        let next = if i == len - 1 {
            variants[0]
        } else {
            variants[i + 1]
        };
        match_arms.push(quote! { #enum_name::#ident => #enum_name::#next });
    }
    let expanded = quote! {
        impl ::data_classes::ToNext for #enum_name {
            fn get_next(&self) -> Self {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    };
    TokenStream::from(expanded)
}

#[cfg(feature = "rand")]
pub fn to_random(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let variants: Vec<&Ident> = input.variants.iter().map(|v| &v.ident).collect();
    let len = variants.len();
    if len == 0 {
        return TokenStream::new();
    }
    let variants = variants.iter().map(|v| quote! { #enum_name::#v });
    let expanded = quote! {
        impl ::data_classes::ToRandom for #enum_name {
            fn random<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
                let choice = rng.gen_range(0..#len);
                match choice {
                    #(x if x == #variants as usize => #variants,)*
                    _ => unreachable!(),
                }
            }
        }
    };
    TokenStream::from(expanded)
}

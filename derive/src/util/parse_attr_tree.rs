use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use syn::{
    Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

struct Name {
    inner: String,
}

impl Parse for Name {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = String::new();
        while !input.is_empty()
            && !input.peek(Token![,])
            && !input.peek(Token![;])
            && !input.peek(syn::token::Paren)
            && !input.peek(syn::token::Bracket)
            && !input.peek(syn::token::Brace)
        {
            let tt: proc_macro2::TokenTree = input.parse()?;
            name.push_str(&tt.to_string());
        }
        Ok(Name { inner: name })
    }
}

struct Node {
    name: Name,
    nodes: Nodes,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let nodes = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            content.parse()?
        } else {
            Nodes {
                inner: Punctuated::new(),
            }
        };
        Ok(Node { name, nodes })
    }
}

struct Nodes {
    inner: Punctuated<Node, Token![,]>,
}

impl Parse for Nodes {
    fn parse(input: ParseStream) -> Result<Self> {
        let nodes = input.parse_terminated(Node::parse, Token![,])?;
        Ok(Nodes { inner: nodes })
    }
}

#[derive(Default)]
pub struct AttrArgs {
    pub nodes: HashMap<String, AttrArgs>,
}

impl AttrArgs {
    pub fn combine(&mut self, other: AttrArgs) {
        for (key, val) in other.nodes {
            if let Some(existing) = self.nodes.get_mut(&key) {
                existing.combine(val);
            } else {
                self.nodes.insert(key, val);
            }
        }
    }
}

impl Display for AttrArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, (key, val)) in self.nodes.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if val.is_empty() {
                write!(f, "{key}")?;
            } else {
                write!(f, "{key}{}", val)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let nodes: Nodes = input.parse()?;
        Ok(AttrArgs::from(nodes))
    }
}

impl From<Nodes> for AttrArgs {
    fn from(nodes: Nodes) -> Self {
        let mut map = HashMap::new();
        for node in nodes.inner {
            if map.contains_key(&node.name.inner) {
                panic!("Duplicate attribute argument: {}", node.name.inner);
            }
            map.insert(node.name.inner, AttrArgs::from(node.nodes));
        }
        AttrArgs { nodes: map }
    }
}

impl Deref for AttrArgs {
    type Target = HashMap<String, AttrArgs>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for AttrArgs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

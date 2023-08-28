use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::parse::Parse;

use crate::{element::Element, value::Value};

/// Possible child nodes inside a component.
#[derive(Debug)]
pub enum Child {
    String(syn::LitStr),
    Braced(syn::ExprBlock),
    Paren(syn::Expr),
    Element(Element),
}

impl Parse for Child {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(value) = input.parse::<Value>() {
            // only allow literals if they are a string.
            match value {
                Value::Lit(lit) => {
                    if let syn::Lit::Str(s) = lit {
                        Ok(Self::String(s))
                    } else {
                        abort!(lit.span(), "only string literals are allowed in children");
                    }
                }
                Value::Block(block) => Ok(Self::Braced(block)),
                Value::Parenthesized(expr) => Ok(Self::Paren(expr)),
            }
        } else if let Ok(elem) = input.parse::<Element>() {
            Ok(Self::Element(elem))
        } else {
            Err(input.error("invalid child"))
        }
    }
}

impl ToTokens for Child {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Child::String(s) => tokens.extend(s.into_token_stream()),
            Child::Braced(b) => tokens.extend(b.into_token_stream()),
            Child::Paren(p) => tokens.extend(quote! {move || #p}),
            Child::Element(e) => tokens.extend(e.into_token_stream()),
        }
    }
}

/// A space-separated series of children.
#[derive(Debug)]
pub struct Children(Vec<Child>);

impl Children {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn children(&self) -> &[Child] {
        &self.0
    }
}

impl Parse for Children {
    /// Does not include the surrounding braces.
    ///
    /// If no children are present, an empty vector will be returned.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();
        while let Ok(child) = input.parse::<Child>() {
            eprintln!("found child {child:?}");
            vec.push(child);
        }
        Ok(Self(vec))
    }
}

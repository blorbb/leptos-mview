use proc_macro_error::abort;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::{derive_multi_ast_for, Element};
use crate::{ast::Value, error_ext::ResultExt, kw};

/// A child that is an actual HTML value (i.e. not a slot).
///
/// Use [`Child`] to try and parse these.
pub enum NodeChild {
    Value(Value),
    Element(Element),
}

impl ToTokens for NodeChild {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Value(v) => tokens.extend(v.into_token_stream()),
            Self::Element(e) => tokens.extend(e.into_token_stream()),
        }
    }
}

/// Possible child items inside a component.
///
/// If the child is a `Value::Lit`, this lit must be a string. Parsing will
/// abort if the lit is not a string.
///
/// Children can either be a [`NodeChild`] (i.e. an actual element), or a slot.
/// Slots are distinguished by prefixing the child with `slot:`.
///
/// # Parsing
/// Mostly **aborts** if parsing fails. An [`Err`] is only returned if there are
/// no tokens remaining.
pub enum Child {
    Node(NodeChild),
    Slot(kw::slot, Element),
}

impl Parse for Child {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(value) = input.parse::<Value>() {
            // only allow literals if they are a string.
            if let Value::Lit(ref lit) = value {
                if let syn::Lit::Str(_) = lit {
                    Ok(Self::Node(NodeChild::Value(value)))
                } else {
                    abort!(lit.span(), "only string literals are allowed in children");
                }
            } else {
                Ok(Self::Node(NodeChild::Value(value)))
            }
        } else if input.peek(kw::slot) && input.peek2(Token![:]) {
            let slot = input.parse::<kw::slot>().unwrap();
            input.parse::<Token![:]>().unwrap();
            let elem = input
                .parse::<Element>()
                .expect_or_abort("expected struct after `slot:`");
            Ok(Self::Slot(slot, elem))
        } else if let Ok(elem) = input.parse::<Element>() {
            Ok(Self::Node(NodeChild::Element(elem)))
        } else {
            Err(input.error("no children remaining"))
        }
    }
}

/// A space-separated series of children.
///
/// Parsing does not include the surrounding braces.
/// If no children are present, an empty vector will be stored.
///
/// There are two ways of passing children, so no `ToTokens` implementation
/// is provided. Use `to_child_methods` or `to_fragment` instead.
pub struct Children(Vec<Child>);
derive_multi_ast_for! {
    struct Children(Vec<Child>);
    impl Parse(non_empty_error = "invalid child: expected literal, block, bracket or element");
}

impl Children {
    pub fn into_vec(self) -> Vec<Child> { self.0 }

    /// Returns an iterator of all children that are not slots.
    pub fn element_children(&self) -> impl Iterator<Item = &NodeChild> {
        self.0.iter().filter_map(|child| match child {
            Child::Node(node) => Some(node),
            Child::Slot(..) => None,
        })
    }

    /// Returns an iterator of all children that are slots.
    pub fn slot_children(&self) -> impl Iterator<Item = &Element> {
        self.0.iter().filter_map(|child| match child {
            Child::Node(_) => None,
            Child::Slot(_, elem) => Some(elem),
        })
    }
}

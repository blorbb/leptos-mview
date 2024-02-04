use proc_macro2::Span;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote, Token,
};

use super::Element;
use crate::{ast::Value, error_ext::ResultExt, kw, recover::rollback_err};

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

impl NodeChild {
    pub fn span(&self) -> Span {
        match self {
            Self::Value(v) => v.span(),
            Self::Element(e) => e.tag().span(),
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
        if let Some(value) = rollback_err(input, Value::parse) {
            // only allow literals if they are a string.
            if let Value::Lit(ref lit) = value {
                if let syn::Lit::Str(_) = lit {
                    Ok(Self::Node(NodeChild::Value(value)))
                } else {
                    emit_error!(lit.span(), "only string literals are allowed in children");
                    Ok(Self::Node(NodeChild::Value(Value::Lit(parse_quote!("")))))
                }
            } else {
                Ok(Self::Node(NodeChild::Value(value)))
            }
        } else if let Some((slot, _)) = rollback_err(input, |input| {
            Ok((kw::slot::parse(input)?, <Token![:]>::parse(input)?))
        }) {
            let elem = input.parse::<Element>()?;
            Ok(Self::Slot(slot, elem))
        } else if input.peek(syn::Ident::peek_any) {
            let elem = Element::parse(input).unwrap_or_abort();
            Ok(Self::Node(NodeChild::Element(elem)))
        } else {
            Err(input.error("no child found"))
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

impl std::ops::Deref for Children {
    type Target = [Child];
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Parse for Children {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();

        while let Some(inner) = rollback_err(input, Child::parse) {
            // proc_macro_error::emit_error!(input.span(), "{:#?}", inner);
            vec.push(inner);
        }

        if !input.is_empty() {
            emit_error!(
                input.span(),
                "invalid child: expected literal, block, bracket or element"
            );
            input.parse::<proc_macro2::TokenStream>().unwrap();
        };

        Ok(Self(vec))
    }
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

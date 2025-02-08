use proc_macro2::Span;
use proc_macro_error2::emit_error;
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote, Token,
};

use super::Element;
use crate::{
    ast::Value,
    error_ext::SynErrorExt,
    kw,
    parse::{self, rollback_err},
};

/// A child that is an actual HTML value (i.e. not a slot).
///
/// Use [`Child`] to try and parse these.
pub enum NodeChild {
    Value(Value),
    Element(Element),
}

impl ToTokens for NodeChild {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let child_tokens = match self {
            Self::Value(v) => v.into_token_stream(),
            Self::Element(e) => e.into_token_stream(),
        };
        tokens.extend(quote! {
            #child_tokens
        });
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
        // make sure its not a fully qualified path
        } else if input.peek(kw::slot) && input.peek2(Token![:]) && !input.peek2(Token![::]) {
            let slot = kw::slot::parse(input).unwrap();
            <Token![:]>::parse(input).unwrap();
            let elem = Element::parse(input)?;
            Ok(Self::Slot(slot, elem))
        } else if input.peek(syn::Ident::peek_any) {
            let elem = Element::parse(input)?;
            Ok(Self::Node(NodeChild::Element(elem)))
        } else {
            Err(input.error("invalid child: expected literal, block, bracket or element"))
        }
    }
}

/// A space-separated series of children.
///
/// Parsing does not include the surrounding braces.
/// If no children are present, an empty vector will be stored.
pub struct Children(Vec<Child>);

impl std::ops::Deref for Children {
    type Target = [Child];
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Parse for Children {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }
            match Child::parse(input) {
                Ok(child) => vec.push(child),
                Err(e) => {
                    if input.peek(Token![;]) {
                        // an extra semi-colon: just skip it and keep parsing
                        emit_error!(
                            e.span(), "extra semi-colon found";
                            help="remove this semi-colon"
                        );
                        <Token![;]>::parse(input).unwrap();
                    } else {
                        e.emit_as_error();
                        // skip the rest of the tokens
                        // need to consume all tokens otherwise an error is made on drop
                        parse::take_rest(input);
                    }
                }
            };
        }

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

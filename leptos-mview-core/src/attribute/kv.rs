use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, Token};

use crate::{error_ext::ResultExt, ident::KebabIdent, value::Value};

/// A `key = value` type of attribute.
///
/// # Examples
/// ```ignore
/// input type="checkbox" data-index=1 checked;
///       ^^^^^^^^^^^^^^^ ^^^^^^^^^^^^
/// ```
/// Directives are not included.
/// ```ignore
/// input on:input={handle_input} type="text";
///       ^^^not included^^^^^^^^ ^included^^
/// ```
///
/// # Parsing
/// If parsing fails, the input `ParseStream` will not be advanced.
///
/// If an identifier and equal sign is found but no value after,
/// the macro will abort.
#[derive(Debug, Clone)]
pub struct KvAttr {
    key: KebabIdent,
    value: Value,
}

impl KvAttr {
    pub const fn key(&self) -> &KebabIdent {
        &self.key
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }

    /// Returns a (key, value) tuple.
    pub const fn kv(&self) -> (&KebabIdent, &Value) {
        (self.key(), self.value())
    }

    /// Converts an attribute to a `.attr(key, value)` token stream.
    pub fn to_attr_method(&self) -> TokenStream {
        let (key, value) = self.kv();
        // handle special cases
        if key.repr() == "ref" {
            quote! { .node_ref(#value) }
        } else {
            quote! { .attr(#key, #value) }
        }
    }

    /// Converts an attribute to a `.key(value)` token stream.
    pub fn to_component_builder_method(&self) -> TokenStream {
        let key = self.key().to_snake_ident();
        let value = self.value();
        quote! {
            .#key(#value)
        }
    }
}

impl Parse for KvAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // first check that this is actually a kv attribute.
        // if there is an ident followed by =, this is kv attribute.
        let fork = input.fork();
        fork.parse::<KebabIdent>()?;

        if fork.peek(Token![=]) {
            // this is a kv attribute: consume main input stream.
            let key = input.parse::<KebabIdent>().unwrap();
            input.parse::<Token![=]>().unwrap();
            let value = input.parse::<Value>().unwrap_or_abort();
            Ok(Self { key, value })
        } else {
            Err(input.error("invalid kv attribute"))
        }
    }
}

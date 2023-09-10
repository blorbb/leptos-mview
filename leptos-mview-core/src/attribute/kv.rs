use syn::{
    parse::{discouraged::Speculative, Parse},
    Token,
};

use crate::{error_ext::ResultExt, ident::KebabIdent, value::Value};

use super::ShorthandAttr;

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
    pub const fn new(key: KebabIdent, value: Value) -> Self {
        Self { key, value }
    }

    pub const fn key(&self) -> &KebabIdent {
        &self.key
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }
}

impl Parse for KvAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // try parse shorthand first
        if input.peek(syn::token::Brace) {
            // don't unwrap_or_abort, as this might be trying to parse
            // the children block instead of an attribute.
            let attr = input.parse::<ShorthandAttr>()?;
            return Ok(Self::new(attr.key, attr.value));
        }

        // check that this is actually a kv attribute.
        // if there is an ident followed by =, this is kv attribute.
        let fork = input.fork();
        let key = fork.parse::<KebabIdent>()?;

        if fork.peek(Token![=]) {
            // this is a kv attribute: consume main input stream.
            input.advance_to(&fork);
            input.parse::<Token![=]>().unwrap();
            let value = input.parse::<Value>().unwrap_or_abort();
            Ok(Self { key, value })
        } else {
            Err(input.error("invalid kv attribute"))
        }
    }
}

use proc_macro2::Span;
use syn::{parse::Parse, parse_quote, Token};

use crate::{
    ast::{BracedKebabIdent, KebabIdent, Value},
    parse::rollback_err,
    span,
};

/// A `key = value` type of attribute.
///
/// This can either be a normal `key = value`, a shorthand `{key}`, or a
/// boolean attribute `checked`.
///
/// # Examples
/// ```ignore
/// input type="checkbox" data-index=1 checked;
///       ^^^^^^^^^^^^^^^ ^^^^^^^^^^^^ ^^^^^^^
/// ```
/// Directives are not included.
/// ```ignore
/// input on:input={handle_input} type="text";
///       ^^^not included^^^^^^^^ ^included^^
/// ```
#[derive(Clone)]
pub struct KvAttr {
    key: KebabIdent,
    value: Value,
}

impl KvAttr {
    pub const fn new(key: KebabIdent, value: Value) -> Self { Self { key, value } }

    pub const fn key(&self) -> &KebabIdent { &self.key }

    pub const fn value(&self) -> &Value { &self.value }

    pub fn span(&self) -> Span { span::join(self.key().span(), self.value().span()) }
}

impl Parse for KvAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (ident, value) = if input.peek(syn::token::Brace) {
            let braced_ident = BracedKebabIdent::parse(input)?;
            (
                braced_ident.ident().clone(),
                braced_ident.into_block_value(),
            )
        } else {
            let ident = KebabIdent::parse(input)?;
            if let Some(eq) = rollback_err(input, <Token![=]>::parse) {
                let value = Value::parse_or_emit_err(input, eq.span);
                (ident, value)
            } else {
                // don't span the attribute name to the `true` or it becomes bool-colored
                let value = Value::Lit(parse_quote!(true));
                (ident, value)
            }
        };

        Ok(Self::new(ident, value))
    }
}

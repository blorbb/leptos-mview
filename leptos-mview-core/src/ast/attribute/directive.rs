use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Token,
};

use crate::{
    ast::{BracedKebabIdent, KebabIdentOrStr, Value},
    parse::rollback_err,
};

/// A special attribute like `on:click={...}`.
///
/// # Examples
/// ```ignore
/// input type="checkbox" on:input={handle_input};
///                       ^^^^^^^^^^^^^^^^^^^^^^^
/// button class:primary={primary} style:color="grey";
///        ^^^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^
/// ```
/// The shorthand syntax is also supported on the argument of directives:
/// ```ignore
/// button class:{primary} style:color="grey";
/// ```
///
/// If an extra `:modifier` is added, there will also be a modifier.
/// ```ignore
/// button on:click:undelegated={on_click};
/// ```
/// `on:{click}:undelegated` also works for the shorthand.
#[derive(Clone)]
pub struct Directive {
    pub(crate) dir: syn::Ident,
    pub(crate) key: KebabIdentOrStr,
    pub(crate) modifier: Option<syn::Ident>, // on:event:undelegated
    pub(crate) value: Option<Value>,
}

impl Parse for Directive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = syn::Ident::parse_any(input)?;
        <Token![:]>::parse(input)?;

        let try_parse_modifier = |input| {
            rollback_err(input, <Token![:]>::parse)
                .is_some()
                .then(|| syn::Ident::parse_any(input))
                .transpose()
        };

        let key: KebabIdentOrStr;
        let value: Option<Value>;
        let modifier: Option<syn::Ident>;

        if input.peek(syn::token::Brace) {
            // on:{click}:undelegated
            let ident = BracedKebabIdent::parse(input)?;
            key = KebabIdentOrStr::KebabIdent(ident.ident().clone());
            value = Some(ident.into_block_value());
            modifier = try_parse_modifier(input)?;
        } else {
            // on:click:undelegated={on_click}
            key = KebabIdentOrStr::parse(input)?;
            modifier = try_parse_modifier(input)?;
            value = rollback_err(input, <Token![=]>::parse)
                .is_some()
                .then(|| Value::parse_or_emit_err(input));
        };

        Ok(Self {
            dir: name,
            key,
            modifier,
            value,
        })
    }
}

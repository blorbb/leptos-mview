use proc_macro2::{Span, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use crate::parse::{extract_braced, rollback_err};

/// A spread attribute like `{..attrs}`.
///
/// The spread after the `..` can be any expression.
#[derive(Clone)]
pub struct SpreadAttr {
    braces: syn::token::Brace,
    dotdot: Token![..],
    rest: TokenStream,
}

impl Parse for SpreadAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // try parse spread attributes `{..attrs}`
        let (braces, stream) = extract_braced(input)?;

        if let Some(dotdot) = rollback_err(&stream, <Token![..]>::parse) {
            let rest = stream.parse::<TokenStream>().unwrap();

            Ok(Self {
                braces,
                dotdot,
                rest,
            })
        } else {
            Err(input.error("invalid spread attribute"))
        }
    }
}

impl SpreadAttr {
    /// Returns the `..` in the spread attr
    pub const fn dotdot(&self) -> &Token![..] { &self.dotdot }

    /// Returns the expression after the `..`.
    pub const fn expr(&self) -> &TokenStream { &self.rest }

    /// Returns the span of the wrapping braces.
    pub fn span(&self) -> Span { self.braces.span.join() }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::SpreadAttr;

    #[test]
    fn compiles() { let _: SpreadAttr = parse_quote!({ ..a }); }
}

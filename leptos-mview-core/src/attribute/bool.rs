use proc_macro2::Span;
use syn::{parse::Parse, parse_quote_spanned};

use crate::ident::KebabIdent;

/// A standalone attribute without a specific value.
///
/// The value is implied to be `true`.
///
/// # Examples
/// ```ignore
/// input type="checkbox" checked;
///                       ^^^^^^^
/// ```
///
/// # Parsing
/// The `parse` method only checks for an identifier. As this is the easiest
/// to match, this attribute should be parsed after all others have been tried.
#[derive(Debug, Clone)]
pub struct BoolAttr(KebabIdent);

impl BoolAttr {
    pub const fn span(&self) -> Span {
        self.0.span()
    }

    pub fn into_key(self) -> KebabIdent {
        self.0
    }

    /// Returns a `true` token spanned to the identifier.
    pub fn spanned_true(&self) -> syn::LitBool {
        parse_quote_spanned!(self.span()=> true)
    }
}

impl Parse for BoolAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse::<KebabIdent>()?))
    }
}

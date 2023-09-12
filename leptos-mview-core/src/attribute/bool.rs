use syn::parse::Parse;

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
    pub fn into_key(self) -> KebabIdent {
        self.0
    }
}

impl Parse for BoolAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse::<KebabIdent>()?))
    }
}

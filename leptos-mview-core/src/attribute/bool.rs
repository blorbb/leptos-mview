use proc_macro2::{Span, TokenStream};
use quote::quote;
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
    pub const fn key(&self) -> &KebabIdent {
        &self.0
    }

    pub const fn span(&self) -> Span {
        self.0.span()
    }

    /// Returns a `true` token spanned to the identifier.
    pub fn spanned_true(&self) -> syn::LitBool {
        parse_quote_spanned!(self.span()=> true)
    }

    /// Returns a key-value pair `(key, true)`, with the `true` being spanned
    /// to the identifier.
    pub fn kv(&self) -> (&KebabIdent, syn::LitBool) {
        (self.key(), self.spanned_true())
    }

    /// Converts an attribute to a `.attr(key, true)` token stream.
    pub fn to_attr_method(&self) -> TokenStream {
        let (key, value) = self.kv();
        quote! { .attr(#key, #value) }
    }

    /// Converts an attribute to a `.key(true)` token stream.
    pub fn to_component_builder_method(&self) -> TokenStream {
        let key = self.key().to_snake_ident();
        let value = self.spanned_true();
        quote! {
            .#key(#value)
        }
    }
}

impl Parse for BoolAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse::<KebabIdent>()?))
    }
}

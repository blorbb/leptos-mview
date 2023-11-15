//! Mini helper functions for working with spans.

use proc_macro2::{Span, TokenStream};
use quote::quote;

/// Tries to join two spans together, returning just the first span if
/// unable to join.
///
/// The spans are unable to join if the user is not on nightly or the spans
/// are in different files.
pub fn join(s1: Span, s2: Span) -> Span { s1.join(s2).unwrap_or(s1) }

/// Gives each span of `spans` the color of a variable.
///
/// Returns an iterator of [`TokenStream`]s that need to be expanded to
/// somewhere in the macro. Each [`TokenStream`] contains `let _ = ();`, so
/// putting it in a block is the easiest way.
pub fn color_all(spans: impl IntoIterator<Item = Span>) -> impl Iterator<Item = TokenStream> {
    spans.into_iter().map(|span| {
        let ident = syn::Ident::new("_", span);
        quote! { let #ident = (); }
    })
}

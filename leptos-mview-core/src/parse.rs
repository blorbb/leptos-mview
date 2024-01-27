//! Mini helper functions for parsing

use syn::parse::{discouraged::Speculative, Parse, ParseStream};

// these functions probably aren't going to change and it's difficult to make
// them generic over the delimiter, so just leaving it with duplication.

/// Parses an AST wrapped in braces.
///
/// Does not advance the token stream if the inner stream does not completely
/// match `T`, including if there are more tokens after the `T`.
pub fn parse_braced<T: Parse>(input: ParseStream) -> syn::Result<(T, syn::token::Brace)> {
    let fork = input.fork();
    if fork.peek(syn::token::Brace) {
        let inner;
        let brace_token = syn::braced!(inner in fork);
        let ast = inner.parse::<T>()?;
        if inner.is_empty() {
            input.advance_to(&fork);
            Ok((ast, brace_token))
        } else {
            Err(inner.error("found extra tokens trying to parse braced expression"))
        }
    } else {
        Err(input.error("no brace found"))
    }
}

/// Parses an AST wrapped in parens.
///
/// Does not advance the token stream if the inner stream does not completely
/// match `T`, including if there are more tokens after the `T`.
pub fn parse_parenthesized<T: Parse>(input: ParseStream) -> syn::Result<(T, syn::token::Paren)> {
    let fork = input.fork();
    if fork.peek(syn::token::Paren) {
        let inner;
        let brace_token = syn::parenthesized!(inner in fork);
        let ast = inner.parse::<T>()?;
        if inner.is_empty() {
            input.advance_to(&fork);
            Ok((ast, brace_token))
        } else {
            Err(inner.error("found extra tokens trying to parse parenthesized expression"))
        }
    } else {
        Err(input.error("no paren found"))
    }
}

//! Mini helper functions for parsing

use proc_macro2::TokenStream;
use syn::parse::{discouraged::Speculative, Parse, ParseBuffer, ParseStream};

pub fn extract_parenthesized(input: ParseStream) -> syn::Result<(syn::token::Paren, ParseBuffer)> {
    let stream;
    let delim = syn::parenthesized!(stream in input);
    Ok((delim, stream))
}

pub fn extract_bracketed(input: ParseStream) -> syn::Result<(syn::token::Bracket, ParseBuffer)> {
    let stream;
    let delim = syn::bracketed!(stream in input);
    Ok((delim, stream))
}

pub fn extract_braced(input: ParseStream) -> syn::Result<(syn::token::Brace, ParseBuffer)> {
    let stream;
    let delim = syn::braced!(stream in input);
    Ok((delim, stream))
}

pub fn bracketed_tokens(input: ParseStream) -> syn::Result<(syn::token::Bracket, TokenStream)> {
    let (delim, buf) = extract_bracketed(input)?;
    let ts = TokenStream::parse(&buf).expect("parsing tokenstream never fails");
    Ok((delim, ts))
}

pub fn braced_tokens(input: ParseStream) -> syn::Result<(syn::token::Brace, TokenStream)> {
    let (delim, buf) = extract_braced(input)?;
    let ts = TokenStream::parse(&buf).expect("parsing tokenstream never fails");
    Ok((delim, ts))
}

// these functions probably aren't going to change and it's difficult to make
// them generic over the delimiter, so just leaving it with duplication.

/// Parses an AST wrapped in braces.
///
/// Does not advance the token stream if the inner stream does not completely
/// match `T`, including if there are more tokens after the `T`.
pub fn braced<T: Parse>(input: ParseStream) -> syn::Result<(syn::token::Brace, T)> {
    let fork = input.fork();
    if fork.peek(syn::token::Brace) {
        let (brace, inner) = extract_braced(&fork).expect("peeked brace");
        let ast = inner.parse::<T>()?;
        if inner.is_empty() {
            input.advance_to(&fork);
            Ok((brace, ast))
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
pub fn parenthesized<T: Parse>(input: ParseStream) -> syn::Result<(syn::token::Paren, T)> {
    let fork = input.fork();
    if fork.peek(syn::token::Paren) {
        let (paren, inner) = extract_parenthesized(&fork).expect("peeked paren");
        let ast = inner.parse::<T>()?;
        if inner.is_empty() {
            input.advance_to(&fork);
            Ok((paren, ast))
        } else {
            Err(inner.error("found extra tokens trying to parse parenthesized expression"))
        }
    } else {
        Err(input.error("no paren found"))
    }
}

pub fn rollback_err<F, T>(input: ParseStream, parser: F) -> Option<T>
where
    F: Fn(ParseStream) -> syn::Result<T>,
{
    let fork = input.fork();
    match parser(&fork) {
        Ok(val) => {
            input.advance_to(&fork);
            Some(val)
        }
        Err(_) => None,
    }
}

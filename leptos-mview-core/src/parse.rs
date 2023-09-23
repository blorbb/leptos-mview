//! Mini helper functions for parsing

use syn::parse::{discouraged::Speculative, ParseStream};

/// Parses an AST wrapped in braces.
///
/// Does not advance the token stream if the inner stream does not completely
/// match `T`.
pub fn parse_braced<T: syn::parse::Parse>(
    input: ParseStream,
) -> syn::Result<(T, syn::token::Brace)> {
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

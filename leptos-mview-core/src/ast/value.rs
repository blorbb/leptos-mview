use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use crate::{parse, recover::rollback_err};

/// Interpolated Rust expressions within the macro.
///
/// Block expressions like `{move || !is_red.get()}` are placed as so.
///
/// Expressions within brackets are wrapped in a closure, e.g. `[!is_red.get()]`
/// is expanded to `{move || !is_red.get()}`.
///
/// Only literals can have no delimiter, to avoid ambiguity.
///
/// Block and bracketed expressions are not parsed as [`syn::Expr`]s as the
/// specific details of what is contained is not required (they are expanded
/// as-is). Instead, a plain [`TokenStream`] is taken, which allows for invalid
/// expressions. rust-analyzer can produce errors at the correct span using this
/// `TokenStream`, and provides better autocompletion (e.g. when looking for
/// methods by entering `something.`).
///
/// # Parsing
/// This AST is considered 'basic', so if parsing fails, an [`Err`] will be
/// returned and it will not advance the [`ParseStream`].
#[derive(Clone)]
pub enum Value {
    Lit(syn::Lit),
    // take a raw `TokenStream` instead of ExprBlock/etc for better r-a support
    // as invalid expressions aren't completely rejected
    Block {
        tokens: TokenStream,
        braces: syn::token::Brace,
    },
    Bracket {
        tokens: TokenStream,
        brackets: syn::token::Bracket,
        prefixes: Option<syn::Ident>,
    },
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let (brackets, tokens) = parse::bracketed_tokens(input).unwrap();
            Ok(Self::Bracket {
                tokens,
                brackets,
                prefixes: None,
            })
        // with prefixes like `f["{}", something]`
        } else if input.peek(syn::Ident::peek_any) && input.peek2(syn::token::Bracket) {
            let prefixes = input.call(syn::Ident::parse_any).unwrap();
            let (brackets, tokens) = parse::bracketed_tokens(input).unwrap();
            Ok(Self::Bracket {
                tokens,
                brackets,
                prefixes: Some(prefixes),
            })
        } else if input.peek(syn::token::Brace) {
            let (braces, tokens) = parse::braced_tokens(input).unwrap();
            Ok(Self::Block { tokens, braces })
        } else if input.peek(syn::Lit) {
            let lit = syn::Lit::parse(input).unwrap();
            Ok(Self::Lit(lit))
        } else {
            Err(input.error("invalid value: expected bracket, block or literal"))
        }
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Self::Lit(lit) => lit.into_token_stream(),
            Self::Block { tokens, braces } => quote_spanned!(braces.span.join()=> {#tokens}),
            Self::Bracket {
                tokens,
                prefixes,
                brackets,
            } => {
                if let Some(prefixes) = prefixes {
                    // only f[] is supported for now
                    if prefixes == "f" {
                        let format = quote_spanned!(prefixes.span()=> format!);
                        quote_spanned!(brackets.span.join()=> move || ::std::#format(#tokens))
                    } else {
                        emit_error!(
                            prefixes.span(),
                            "unsupported prefix: only `f` is supported."
                        );
                        quote! {}
                    }
                } else {
                    quote_spanned!(brackets.span.join()=> move || #tokens)
                }
            }
        });
    }
}

impl Value {
    /// Returns the [`Span`] of this [`Value`].
    ///
    /// If the value is a block/bracket, the span includes the delimiters.
    pub fn span(&self) -> Span {
        match self {
            Self::Lit(lit) => lit.span(),
            Self::Block { braces, .. } => braces.span.join(),
            Self::Bracket { brackets, .. } => brackets.span.join(),
        }
    }

    /// Either parses a valid [`Value`], or inserts a `MissingValueAfterEq`
    /// never-type enum.
    pub fn parse_or_never(input: ParseStream) -> Self {
        if let Some(value) = rollback_err(input, Self::parse) {
            value
        } else {
            // incomplete typing; place a MissingValueAfterEq and continue
            emit_error!(input.span(), "expected value after =");
            Self::Block {
                tokens: quote_spanned!(input.span() => ::leptos_mview::MissingValueAfterEq),
                braces: syn::token::Brace(input.span()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Value;

    /// Variant-only version of `Value` for quick checking.
    enum ValueKind {
        Lit,
        Block,
        Bracket,
    }

    // test only implementation, as it is not used anywhere else.
    impl Value {
        pub fn is_lit(&self) -> bool { matches!(self, Self::Lit(_)) }

        pub fn is_block(&self) -> bool { matches!(self, Self::Block { .. }) }

        pub fn is_bracketed(&self) -> bool { matches!(self, Self::Bracket { .. }) }
    }

    impl ValueKind {
        fn value_is(&self, value: Value) -> bool {
            match self {
                ValueKind::Lit => value.is_lit(),
                ValueKind::Block => value.is_block(),
                ValueKind::Bracket => value.is_bracketed(),
            }
        }
    }

    #[test]
    fn value_conversion() {
        let mut exprs = HashMap::new();

        exprs.insert("\"hi\"", ValueKind::Lit);
        exprs.insert("1", ValueKind::Lit);
        exprs.insert("true", ValueKind::Lit);
        exprs.insert("{value}", ValueKind::Block);
        exprs.insert("{value; value2; value3}", ValueKind::Block);
        exprs.insert("[abc.get()]", ValueKind::Bracket);
        exprs.insert("{(aa,)}", ValueKind::Block);
        exprs.insert("[{a; b}]", ValueKind::Bracket);

        for (expr, kind) in exprs {
            let value = syn::parse_str(expr).unwrap();
            assert!(kind.value_is(value))
        }
    }
}

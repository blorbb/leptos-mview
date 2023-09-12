use proc_macro2::Span;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};

/// Interpolated values.
/// Plain expressions or block expressions like `{move || !is_red.get()}`
/// are placed as so.
///
/// Expressions within parens are wrapped in a closure, e.g. `(!is_red.get())`
/// is expanded to `{move || !is_red.get()}`.
///
/// To use tuples, wrap it in braces or parens, e.g. `{(1, 2)}` or `((1, 2))`.
///
/// Only literals can have the `None` delimiter, to avoid ambiguity.
#[derive(Debug, Clone)]
pub enum Value {
    Lit(syn::Lit),
    Block(syn::ExprBlock),
    Parenthesized(syn::Expr, syn::token::Paren),
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let stream;
            let parens = syn::parenthesized!(stream in input);

            // fork to show better errors.
            let full_stream = stream.fork();
            let expr: syn::Expr = stream.parse()?;
            // parsed an expression but there is still more.
            if stream.is_empty() {
                Ok(Self::Parenthesized(expr, parens))
            } else {
                abort!(
                    stream.span(), "unexpected token";
                    note = "\
                    if trying to pass a tuple, wrap it in additional braces or \
                    parens. e.g. (({})) or {{({})}}.\
                    ", full_stream, full_stream
                )
            }
        } else if input.peek(syn::token::Brace) {
            Ok(Self::Block(input.parse()?))
        } else if let Ok(lit) = input.parse::<syn::Lit>() {
            Ok(Self::Lit(lit))
        } else {
            Err(input.error("invalid value: expected paren, block or literal"))
        }
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Self::Lit(lit) => lit.into_token_stream(),
            Self::Block(block) => block.into_token_stream(),
            Self::Parenthesized(expr, _) => quote! {move || #expr},
        });
    }
}

impl Value {
    /// Tries to convert a `Value` into a single ident.
    ///
    /// Example: the value `{something}` becomes the ident `something`.
    ///
    /// Returns `None` if the block does not only contain an ident.
    pub fn as_block_with_ident(&self) -> Option<&syn::Ident> {
        let Self::Block(block) = self else {
            return None;
        };
        if block.block.stmts.len() != 1 {
            return None;
        }
        let syn::Stmt::Expr(syn::Expr::Path(path), None) = block.block.stmts.first()? else {
            return None;
        };

        path.path.get_ident()
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Lit(lit) => lit.span(),
            Self::Block(block) => block.block.brace_token.span.join(),
            Self::Parenthesized(_, parens) => parens.span.join(),
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
        Paren,
    }

    // test only implementation, as it is not used anywhere else.
    impl Value {
        pub fn is_lit(&self) -> bool {
            matches!(self, Self::Lit(_))
        }

        pub fn is_block(&self) -> bool {
            matches!(self, Self::Block(_))
        }

        pub fn is_parenthesized(&self) -> bool {
            matches!(self, Self::Parenthesized(..))
        }
    }

    impl ValueKind {
        fn value_is(&self, value: Value) -> bool {
            match self {
                ValueKind::Lit => value.is_lit(),
                ValueKind::Block => value.is_block(),
                ValueKind::Paren => value.is_parenthesized(),
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
        exprs.insert("(abc.get())", ValueKind::Paren);
        exprs.insert("{(aa,)}", ValueKind::Block);
        exprs.insert("({a; b})", ValueKind::Paren);

        for (expr, kind) in exprs {
            let value = syn::parse_str(expr).unwrap();
            assert!(kind.value_is(value))
        }
    }
}

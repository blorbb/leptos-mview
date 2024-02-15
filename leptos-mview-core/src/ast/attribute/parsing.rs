//! A collection of structs and functions for parsing attributes.

use proc_macro2::Span;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    token::Brace,
    Token,
};

use crate::{
    ast::{KebabIdent, Value},
    parse,
    recover::rollback_err,
};

/// Parsing function for attributes that can accept:
/// - Normal `key={value}` pairs
/// - Shorthand attributes like `{class}` to `class={class}`
/// - Boolean attributes like `checked` to `checked=true`
/// - The above can also be kebab-case idents.
///
/// For use with `attr` directives and key-value attributes.
pub fn parse_kebab_or_braced_or_bool(input: ParseStream) -> syn::Result<(KebabIdent, Value)> {
    if input.peek(syn::token::Brace) {
        let braced_ident = input.parse::<BracedKebabIdent>()?;
        Ok((
            braced_ident.ident().clone(),
            braced_ident.into_block_value(),
        ))
    } else {
        let ident = KebabIdent::parse(input)?;
        if rollback_err(input, <Token![=]>::parse).is_some() {
            let value = Value::parse_or_emit_err(input);
            Ok((ident, value))
        } else {
            let value = Value::Lit(parse_quote!(true));
            Ok((ident, value))
        }
    }
}

// Parse either a kebab-case ident or a str literal.
#[derive(Clone)]
pub enum KebabIdentOrStr {
    KebabIdent(KebabIdent),
    Str(syn::LitStr),
}

impl KebabIdentOrStr {
    pub fn to_lit_str(&self) -> syn::LitStr {
        match self {
            Self::KebabIdent(ident) => ident.to_lit_str(),
            Self::Str(s) => s.clone(),
        }
    }

    pub fn to_ident_or_emit(&self) -> syn::Ident {
        match self {
            KebabIdentOrStr::KebabIdent(i) => i.to_snake_ident(),
            KebabIdentOrStr::Str(s) => {
                emit_error!(s.span(), "expected identifier");
                syn::Ident::new("__invalid_identifier_found_str", s.span())
            }
        }
    }

    pub fn span(&self) -> Span {
        match self {
            KebabIdentOrStr::KebabIdent(k) => k.span(),
            KebabIdentOrStr::Str(s) => s.span(),
        }
    }
}

impl Parse for KebabIdentOrStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(str) = input.parse::<syn::LitStr>() {
            Ok(Self::Str(str))
        } else {
            Ok(Self::KebabIdent(input.parse()?))
        }
    }
}

/// Parses a braced kebab-cased ident like `{abc-123}`
///
/// Does not advance the token stream if it cannot be parsed.
pub struct BracedKebabIdent {
    brace_token: Brace,
    ident: KebabIdent,
}

impl BracedKebabIdent {
    pub const fn new(brace: Brace, ident: KebabIdent) -> Self {
        Self {
            brace_token: brace,
            ident,
        }
    }

    pub const fn ident(&self) -> &KebabIdent { &self.ident }

    pub fn into_block_value(self) -> Value {
        Value::Block {
            tokens: self.ident().to_snake_ident().into_token_stream(),
            braces: self.brace_token,
        }
    }
}

impl Parse for BracedKebabIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (brace, ident) = parse::braced::<KebabIdent>(input)?;
        Ok(Self::new(brace, ident))
    }
}

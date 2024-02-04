//! A collection of structs and functions for parsing attributes.

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
            let value = Value::parse_or_never(input);
            Ok((ident, value))
        } else {
            let value = Value::Lit(parse_quote!(true));
            Ok((ident, value))
        }
    }
}

/// Parsing function for attributes that can accept:
/// - Normal `key={value}` pairs
/// - Keys that are str literals `"something"={value}`
/// - Shorthand attributes like `{class}` to `class={class}`.
/// - The above can also be kebab-case idents.
pub fn parse_kebab_or_braced_or_str(input: ParseStream) -> syn::Result<(syn::LitStr, Value)> {
    // either a shorthand `{class}` or key-value pair `class={class}`.
    if input.peek(syn::token::Brace) {
        let braced_ident = input.parse::<BracedKebabIdent>()?;
        Ok((
            braced_ident.ident().to_lit_str(),
            braced_ident.into_block_value(),
        ))
    } else {
        let class = KebabIdentOrStr::parse(input)?.into_lit_str();
        <Token![=]>::parse(input)?;
        let value = Value::parse_or_never(input);
        Ok((class, value))
    }
}

/// Parsing functions for attributes that can accept:
/// - Normal `key={value}` pairs
/// - Shorthand attributes like `{class}` to `class={class}`
/// - All idents must be a regular ident, cannot be a keyword.
pub fn parse_ident_or_braced(input: ParseStream) -> syn::Result<(syn::Ident, Value)> {
    if input.peek(syn::token::Brace) {
        let ident = BracedIdent::parse(input)?;
        Ok((ident.ident().clone(), ident.into_block_value()))
    } else {
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<Token![=]>()?;
        let value = Value::parse_or_never(input);
        Ok((ident, value))
    }
}

/// Parsing functions for attributes that can accept:
/// - Normal `key={value}` pairs
/// - Shorthand attributes like `{class}` to `class={class}`
/// - Just an ident, no value afterward.
/// - All idents must be a regular ident, cannot be a keyword.
pub fn parse_ident_optional_value(input: ParseStream) -> syn::Result<(syn::Ident, Option<Value>)> {
    if input.peek(syn::token::Brace) {
        let ident = BracedIdent::parse(input)?;
        Ok((ident.ident().clone(), Some(ident.into_block_value())))
    } else {
        let ident = syn::Ident::parse(input)?;
        if rollback_err(input, <Token![=]>::parse).is_some() {
            // add a value
            let value = Value::parse_or_never(input);
            Ok((ident, Some(value)))
        } else {
            // no value
            Ok((ident, None))
        }
    }
}

/// Generic parsing function for directives.
///
/// Tries the parse the `Kw` and colon, then parses the `next` function.
pub fn parse_dir_then<Kw: syn::token::Token + Parse, R>(
    input: ParseStream,
    next: fn(ParseStream) -> syn::Result<R>,
) -> syn::Result<(Kw, R)> {
    let dir = Kw::parse(input)?; // should not advance if no match
    <Token![:]>::parse(input)?;
    Ok((dir, next(input)?))
}

// Parse either a kebab-case ident or a str literal.
pub enum KebabIdentOrStr {
    KebabIdent(KebabIdent),
    Str(syn::LitStr),
}

impl KebabIdentOrStr {
    pub fn into_lit_str(self) -> syn::LitStr {
        match self {
            Self::KebabIdent(ident) => ident.to_lit_str(),
            Self::Str(s) => s,
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

/// Parses a braced ident like `{abc_123}`
///
/// Does not advance the token stream if it cannot be parsed.
///
/// Does not parse kebab-case identifiers - see [`BracedKebabIdent`] instead.
pub struct BracedIdent {
    brace_token: Brace,
    ident: syn::Ident,
}

impl BracedIdent {
    pub const fn new(brace: Brace, ident: syn::Ident) -> Self {
        Self {
            brace_token: brace,
            ident,
        }
    }

    pub const fn ident(&self) -> &syn::Ident { &self.ident }

    pub fn into_block_value(self) -> Value {
        Value::Block {
            tokens: self.ident().into_token_stream(),
            braces: self.brace_token,
        }
    }
}

impl Parse for BracedIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (brace, ident) = parse::braced::<syn::Ident>(input)?;
        Ok(Self::new(brace, ident))
    }
}

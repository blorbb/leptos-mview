use std::hash::Hash;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    token::Brace,
    Token,
};

use super::Value;
use crate::{parse, span};

/// A kebab-cased identifier.
///
/// The identifier must start with a letter, underscore or dash. The rest of
/// the identifier can have numbers as well. Rust keywords are also allowed.
///
/// Because whitespace is ignored in macros, and a dash is usually interpreted
/// as subtraction, spaces between each segment is allowed but will be ignored.
///
/// Valid [`KebabIdent`]s include `one`, `two-bits`, `--css-variable`,
/// `blue-100`, `-0`, `--a---b_c`, `_a`; but does not include `3d-thing`.
///
/// Equality and hashing are implemented and only based on the repr, not the
/// spans.
///
/// # Parsing
/// If the next token is not a `-` or ident, an [`Err`] is returned and the
/// [`ParseStream`] is not advanced. Otherwise, parsing will stop once the ident
/// ends, and the `ParseStream` is advanced to after this kebab-ident.
///
/// # Expanding
/// The default [`ToTokens`] implementation expands this to a string literal
/// with the appropriate [`Span`]. If a [`syn::Ident`] is desired, use
/// [`Self::to_snake_ident`] instead.
///
/// # Invariants
/// The [`repr`](Self::repr) and [`spans`](Self::spans) fields are not empty. To
/// construct a new [`KebabIdent`], use the [`From<proc_macro2::Ident>`]
/// implementation or parse one with the [`Parse`] implementation.
#[derive(Clone)]
pub struct KebabIdent {
    repr: String,
    spans: Vec<Span>,
}

impl KebabIdent {
    /// Returns a reference to the repr of this [`KebabIdent`].
    pub fn repr(&self) -> &str { self.repr.as_ref() }

    /// Returns the span of this [`KebabIdent`].
    ///
    /// The span of the first and last 'section' (dash, ident or lit int) are
    /// joined. This only works on nightly, so only the first section's span is
    /// returned on stable.
    pub fn span(&self) -> Span {
        span::join(
            self.spans[0],
            *self.spans.last().expect("kebab ident should not be empty"),
        )
    }

    /// Returns an iterator of every span in this [`KebabIdent`].
    ///
    /// Spans usually need to be owned, so an iterator that produces owned spans
    /// is returned.
    pub fn spans(&self) -> impl ExactSizeIterator<Item = Span> + '_ { self.spans.iter().copied() }

    /// Converts this ident to a `syn::LitStr` of the ident's repr with the
    /// appropriate span.
    pub fn to_lit_str(&self) -> syn::LitStr { syn::LitStr::new(self.repr(), self.span()) }

    /// Expands this ident to its string literal, along with dummy items to make
    /// each segment the same color as a variable.
    ///
    /// **NOTE:** The string itself won't be spanned to this [`KebabIdent`].
    /// Make sure that where this is used will always take a string and never
    /// errors.
    ///
    /// The [`TokenStream`] returned is a block expression, so make sure that
    /// blocks can be used in the context where this is expanded.
    pub fn to_str_colored(&self) -> TokenStream {
        let dummy_items = span::color_all(self.spans());
        let string = self.repr();
        quote! {
            {#(#dummy_items)* #string}
        }
    }

    /// Converts this ident to a `syn::Ident` with the appropriate span, by
    /// replacing all `-`s with `_`.
    ///
    /// The span will only be the first 'section' on stable, but correctly
    /// covers the full ident on nightly. See [`KebabIdent::span`] for more
    /// details.
    pub fn to_snake_ident(&self) -> syn::Ident {
        let snake_string = self.repr().replace('-', "_");
        // This will always be valid as the first 'section' must be a `-` or rust ident,
        // which means it starts with `_` or another valid identifier beginning. The int
        // literals within the ident (e.g. between `-`s, like `blue-100`) are allowed
        // since the ident does not start with a number.
        syn::Ident::new(&snake_string, self.span())
    }
}

impl Parse for KebabIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut repr = String::new();
        let mut spans = Vec::new();

        // Start with `-` or letter.
        if let Ok(ident) = input.call(syn::Ident::parse_any) {
            repr.push_str(&ident.to_string());
            spans.push(ident.span());
        } else if let Ok(dash) = input.parse::<Token![-]>() {
            repr.push('-');
            spans.push(dash.span);
        } else {
            return Err(input.error("expected a kebab-cased ident"));
        };

        // Whether we are parsing the second token now.
        // Can't just check if `repr == "-"` as it will cause an infinite
        // loop if the ident is only `-`.
        let mut is_second_token = true;

        // Parse any `-` and idents.
        loop {
            // After every loop, the next ident should be a `-`.
            // Otherwise, this means it was two idents separated by a space,
            // e.g. `one two`.
            if input.parse::<Token![-]>().is_ok() {
                repr.push('-');
            } else if !(is_second_token && repr == "-") {
                // unless the ident starts with a single `-`, then the next
                // token can be an ident or number.
                break;
            }

            is_second_token = false;

            // add ident or number
            if let Ok(ident) = input.call(syn::Ident::parse_any) {
                repr.push_str(&ident.to_string());
                spans.push(ident.span());
            } else if let Ok(int) = input.parse::<syn::LitInt>() {
                repr.push_str(&int.to_string());
                spans.push(int.span());
            };
        }

        // both repr and spans are not empty due to the first-segment check
        Ok(Self { repr, spans })
    }
}

impl ToTokens for KebabIdent {
    /// The identifier will be most often used as a string, so the default
    /// implementation adds an appropriately spanned string.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let repr = self.repr();
        tokens.extend(quote_spanned!(self.span()=> #repr));
    }
}

impl From<proc_macro2::Ident> for KebabIdent {
    fn from(value: proc_macro2::Ident) -> Self {
        // repr is not empty as `proc_macro2::Ident` must be a valid Rust identifier,
        // and "" is not.
        Self {
            repr: value.to_string(),
            spans: vec![value.span()],
        }
    }
}

// eq and hash are only based on the repr

impl PartialEq for KebabIdent {
    fn eq(&self, other: &Self) -> bool { self.repr == other.repr }
}

impl Eq for KebabIdent {}

impl Hash for KebabIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.repr.hash(state); }
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
            Self::KebabIdent(i) => i.to_snake_ident(),
            Self::Str(s) => {
                emit_error!(s.span(), "expected identifier");
                syn::Ident::new("__invalid_identifier_found_str", s.span())
            }
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
/// Equivalent to `parse::braced::<KebabIdent>(input)`, but provides a few
/// methods to help with conversions.
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
            tokens: self.ident.to_snake_ident().into_token_stream(),
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

#[cfg(test)]
mod tests {
    use super::KebabIdent;

    #[test]
    fn valid_reprs() {
        let streams = [
            "word",
            "two-words",
            "--var-abc",
            "-a-b",
            "let--a",
            "struct-b-",
            "blue-100",
            "blue-100a",
            "number-0xa1b2",
            "-",
            "-_-_a",
            "for",
        ];

        for stream in streams {
            let ident: KebabIdent = syn::parse_str(stream).unwrap();
            assert_eq!(ident.repr(), stream)
        }
    }

    #[test]
    fn invalid_reprs() {
        let streams = ["data-thing- =", "distinct idents"];

        for stream in streams {
            let ident = syn::parse_str::<KebabIdent>(stream);
            assert!(ident.is_err());
        }
    }

    #[test]
    fn different_reprs() {
        let streams = ["two - words", "- - a - b"];

        for stream in streams {
            let ident = syn::parse_str::<KebabIdent>(stream).unwrap();
            assert_eq!(ident.repr(), stream.replace(' ', ""));
        }
    }
}

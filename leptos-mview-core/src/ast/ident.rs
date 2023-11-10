use std::hash::Hash;

use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Token,
};

use crate::span;

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

    /// Converts this ident to a `syn::LitStr` of the ident's repr with the
    /// appropriate span.
    pub fn to_lit_str(&self) -> syn::LitStr { syn::LitStr::new(self.repr(), self.span()) }

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
            return Err(input.error("input is not a kebab-cased ident"));
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

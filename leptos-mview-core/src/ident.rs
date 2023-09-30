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
/// The identifier must start with a letter, underscore or dash.
/// The rest of the identifier can have numbers as well.
#[derive(Debug, Clone)]
pub struct KebabIdent {
    repr: String,
    spans: Vec<Span>,
}

impl KebabIdent {
    /// Both `repr` and `spans` should not be empty.
    const fn new(repr: String, spans: Vec<Span>) -> Self { Self { repr, spans } }

    pub fn repr(&self) -> &str { self.repr.as_ref() }

    pub fn to_lit_str(&self) -> syn::LitStr { syn::LitStr::new(self.repr(), self.span()) }

    pub fn span(&self) -> Span {
        span::join(
            self.spans[0],
            *self.spans.last().expect("kebab ident should not be empty"),
        )
    }

    pub fn to_snake_ident(&self) -> syn::Ident {
        let snake_string = self.repr().replace('-', "_");
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

        Ok(Self::new(repr, spans))
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

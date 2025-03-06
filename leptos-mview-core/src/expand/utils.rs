use proc_macro_error2::{abort, emit_error};
use syn::{ext::IdentExt, parse_quote, spanned::Spanned};

#[allow(clippy::doc_markdown)]
// just doing a manual implementation as theres only one need for this (slots).
// Use the `paste` crate if more are needed in the future.
/// `ident` must be an UpperCamelCase word with only ascii word characters.
pub fn upper_camel_to_snake_case(ident: &str) -> String {
    let mut new = String::with_capacity(ident.len());
    // all characters should be ascii
    for char in ident.chars() {
        // skip the first `_`.
        if char.is_ascii_uppercase() && !new.is_empty() {
            new.push('_');
        };
        new.push(char.to_ascii_lowercase());
    }

    new
}

pub fn snake_case_to_upper_camel(ident: syn::Ident) -> syn::Ident {
    let str = ident.unraw().to_string();
    let mut new = String::with_capacity(str.len());
    let mut next_char_is_word_start = true;

    for char in str.chars() {
        match (char, next_char_is_word_start) {
            ('_', _) => {
                next_char_is_word_start = true;
            }
            (c, true) => {
                next_char_is_word_start = false;
                new.extend(c.to_uppercase());
            }
            (c, false) => {
                new.push(c);
            }
        }
    }

    syn::Ident::new_raw(&new, ident.span())
}

pub fn emit_error_if_modifier(m: Option<&syn::Ident>) {
    if let Some(modifier) = m {
        emit_error!(
            modifier.span(),
            "unknown modifier: modifiers are only supported on `on:` directives"
        );
    }
}

/// Converts a [`syn::Path`] (which could include things like `Vec<i32>`) to
/// always use the turbofish (like `Vec::<i32>`).
pub fn turbofishify(mut path: syn::Path) -> syn::Path {
    path.segments
        .iter_mut()
        .for_each(|segment| match &mut segment.arguments {
            syn::PathArguments::None => (),
            syn::PathArguments::AngleBracketed(generics) => {
                generics.colon2_token.get_or_insert(parse_quote!(::));
            }
            // this would probably never happen, not caring about recoverability.
            syn::PathArguments::Parenthesized(p) => {
                abort!(p.span(), "function generics are not allowed")
            }
        });
    path
}

#[cfg(test)]
mod tests {
    use quote::{quote, ToTokens};

    use super::turbofishify;

    #[test]
    fn add_turbofish() {
        let path = syn::parse2::<syn::Path>(quote! { std::vec::Vec<i32> }).unwrap();
        let path = turbofishify(path);
        assert_eq!(
            "std::vec::Vec::<i32>",
            path.to_token_stream().to_string().replace(' ', "")
        );
    }

    #[test]
    fn leave_turbofish() {
        let path = syn::parse2::<syn::Path>(quote! { std::vec::Vec::<i32> }).unwrap();
        let path = turbofishify(path);
        assert_eq!(
            "std::vec::Vec::<i32>",
            path.to_token_stream().to_string().replace(' ', "")
        );
    }
}

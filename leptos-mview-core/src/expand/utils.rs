use proc_macro_error::emit_error;

#[allow(clippy::doc_markdown)]
// just doing a manual implementation as theres only one need for this (slots).
// Use the `paste` crate if more are needed in the future.
/// `ident` must be an UpperCamelCase word with only ascii word characters.
pub(crate) fn upper_camel_to_snake_case(ident: &str) -> String {
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

pub fn emit_error_if_modifier(m: Option<&syn::Ident>) {
    if let Some(modifier) = m {
        emit_error!(
            modifier.span(),
            "modifiers are only supported on `on:` directives"
        );
    }
}

pub mod bool;
pub mod directive;
pub mod kv;

use std::ops::Deref;

use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_quote,
};

use self::{bool::BoolAttr, directive::DirectiveAttr, kv::KvAttr};
use crate::{ident::KebabIdent, value::Value};

/// Parses a shorthand attribute like `{class}`.
///
/// The `key` is the kebab-cased identifier in the braces, and the `value`
/// will always be a `syn::ExprBlock` with a single ident inside.
///
/// # Examples
/// ```ignore
/// let class = "these are classes";
/// let aria_label = "good label here";
/// let (value, set_value) = create_signal(String::new())
/// view! {
///     input type="text" {class} {aria-label} prop:{value};
/// }
/// // is the same as:
/// view! {
///     input type="text" class={class} aria-label={aria_label} prop:value={value};
/// }
/// ```
///
/// # Aborts
/// Returns an `Err` if no brace is found. If a brace is found but the
/// block is not an ident, the macro will abort.
struct ShorthandAttr {
    key: KebabIdent,
    value: Value,
}

impl Parse for ShorthandAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Brace) {
            let fork = input.fork();
            let inner;
            syn::braced!(inner in fork);

            // inner must be a kebab ident
            let ident = inner.parse::<KebabIdent>()?;
            if !inner.is_empty() {
                return Err(input.error("unexpected token after ident"));
            };

            let ident_snake = ident.to_snake_ident();
            let block: syn::ExprBlock = parse_quote!({#ident_snake});

            // advance the actual stream
            input.advance_to(&fork);

            Ok(Self {
                key: ident,
                value: Value::Block(block),
            })
        } else {
            Err(input.error("expected braces for attribute shorthand"))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Attr {
    Kv(KvAttr),
    Bool(BoolAttr),
    Directive(DirectiveAttr),
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(kv) = input.parse::<KvAttr>() {
            Ok(Self::Kv(kv))
        } else if let Ok(dir) = input.parse::<DirectiveAttr>() {
            Ok(Self::Directive(dir))
        } else if let Ok(bool) = input.parse::<BoolAttr>() {
            Ok(Self::Bool(bool))
        } else {
            Err(input.error("no attribute found"))
        }
    }
}

/// A simplified equivalent version of `Attr`.
///
/// Currently this only reduces `BoolAttr`s into a `KvAttr` with a
/// value of `true`.
///
/// TODO: When field shorthands are supported, this will simplify that
/// into the expanded form as well.
#[derive(Debug, Clone)]
pub enum SimpleAttr {
    Kv(KvAttr),
    Directive(DirectiveAttr),
}

impl From<Attr> for SimpleAttr {
    fn from(value: Attr) -> Self {
        match value {
            Attr::Kv(kv) => Self::Kv(kv),
            Attr::Bool(b) => {
                // don't span with the original ident, syntax highlighting
                // looks bad with the boolean color
                let value: syn::Lit = parse_quote!(true);
                Self::Kv(KvAttr::new(b.into_key(), Value::Lit(value)))
            }
            Attr::Directive(dir) => Self::Directive(dir),
        }
    }
}

/// A space-separated series of attributes.
#[derive(Debug, Clone)]
pub struct Attrs(Vec<Attr>);

impl Deref for Attrs {
    type Target = [Attr];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Attrs {
    pub fn into_vec(self) -> Vec<Attr> {
        self.0
    }
}

impl Parse for Attrs {
    /// If no attributes are present, an empty vector will be returned.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();
        while let Ok(attr) = input.parse::<Attr>() {
            vec.push(attr);
        }
        Ok(Self(vec))
    }
}

#[derive(Debug, Clone)]
pub struct SimpleAttrs(Vec<SimpleAttr>);

impl From<Attrs> for SimpleAttrs {
    fn from(value: Attrs) -> Self {
        Self(value.into_vec().into_iter().map(Into::into).collect())
    }
}

impl Parse for SimpleAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.parse::<Attrs>()?;
        Ok(attrs.into())
    }
}

impl Deref for SimpleAttrs {
    type Target = [SimpleAttr];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::attribute::{Attrs, BoolAttr};

    use super::{Attr, KvAttr};

    impl Attr {
        pub fn as_kv(&self) -> Option<&KvAttr> {
            if let Self::Kv(v) = self {
                Some(v)
            } else {
                None
            }
        }

        pub fn as_bool(&self) -> Option<&BoolAttr> {
            if let Self::Bool(v) = self {
                Some(v)
            } else {
                None
            }
        }
    }

    impl Attrs {
        pub fn as_slice(&self) -> &[Attr] {
            &self.0
        }
    }

    #[track_caller]
    fn check_kv(input: &str, output: KvAttr) {
        let (key, value) = input.split_once('=').unwrap();

        assert_eq!(output.key().repr(), key.trim());
        if value.contains('{') {
            assert!(output.value().is_block());
        } else if value.contains('(') {
            assert!(output.value().is_parenthesized());
        } else {
            assert!(output.value().is_lit());
        }
    }

    #[track_caller]
    fn check_bool(input: &str, output: BoolAttr) {
        assert_eq!(output.into_key().repr(), input);
    }

    #[test]
    fn parse_kv_attr() {
        // must not have a mix of parens and braces
        let inputs = ["key = \"value\"", "something-else = {move || true}"];

        for input in inputs {
            let attr = syn::parse_str::<KvAttr>(input).unwrap();
            check_kv(input, attr);
        }
    }

    #[test]
    fn parse_bool_attr() {
        let inputs = ["key", "something-else", "-a-b-c"];
        for input in inputs {
            let attr = syn::parse_str::<BoolAttr>(input).unwrap();
            check_bool(input, attr);
        }
    }

    #[test]
    fn parse_complex_attrs() {
        let input = "key1=(value1) key2 key3 key4={value4}";
        let inputs: Vec<_> = input.split_whitespace().collect();
        let attrs = syn::parse_str::<Attrs>(input).unwrap();
        let attrs = attrs.as_slice();

        check_kv(inputs[0], attrs[0].as_kv().unwrap().clone());
        check_bool(inputs[1], attrs[1].as_bool().unwrap().clone());
        check_bool(inputs[2], attrs[2].as_bool().unwrap().clone());
        check_kv(inputs[3], attrs[3].as_kv().unwrap().clone());
    }
}

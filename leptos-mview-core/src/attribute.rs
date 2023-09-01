use core::slice;

use proc_macro_error::abort;
use syn::{parse::Parse, Token};

use crate::{ident::KebabIdent, value::Value};

/// A `key = value` type of attribute.
///
/// Example:
/// ```text
/// <input type="checkbox" checked />
///        ^^^^^^^^^^^^^^^
/// ```
#[derive(Debug, Clone)]
pub struct KvAttr {
    key: KebabIdent,
    equals_token: Token![=],
    value: Value,
}

impl KvAttr {
    pub fn key(&self) -> &KebabIdent {
        &self.key
    }

    pub fn equals_token(&self) -> syn::token::Eq {
        self.equals_token
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Parse for KvAttr {
    /// Parses a ParseStream into a KvAttr.
    ///
    /// Does not fork the ParseStream; if parsing fails, part of the stream
    /// will be consumed. Fork the stream before parsing if needed.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse::<KebabIdent>()?;
        let equals_token = input.parse::<Token![=]>()?;
        // if the equals token has been parsed but the next token fails,
        // then the value is incorrect, should abort.
        let value = input
            .parse::<Value>()
            .unwrap_or_else(|e| abort!(e.span(), e.to_string()));

        Ok(Self {
            key,
            equals_token,
            value,
        })
    }
}

/// A standalone attribute without a specific value.
///
/// Example:
/// ```text
/// <input type="checkbox" checked />
///                        ^^^^^^^
/// ```
#[derive(Debug, Clone)]
pub struct BoolAttr(KebabIdent);

impl BoolAttr {
    pub fn key(&self) -> &KebabIdent {
        &self.0
    }

    pub fn span(&self) -> proc_macro2::Span {
        self.0.span()
    }
}

impl Parse for BoolAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse::<KebabIdent>()?))
    }
}

#[derive(Debug, Clone)]
pub enum Attr {
    Kv(KvAttr),
    Bool(BoolAttr),
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // stream may be advanced if KvAttr parse fails
        let fork = input.fork();

        if let Ok(kv) = fork.parse::<KvAttr>() {
            input.parse::<KvAttr>().unwrap();
            Ok(Self::Kv(kv))
        } else {
            let bool_attr = input.parse::<BoolAttr>()?;

            Ok(Self::Bool(bool_attr))
        }
    }
}

impl Attr {
    /// Returns `true` if the attr is [`Kv`].
    ///
    /// [`Kv`]: Attr::Kv
    #[must_use]
    pub fn is_kv(&self) -> bool {
        matches!(self, Self::Kv(..))
    }

    /// Returns `true` if the attr is [`Bool`].
    ///
    /// [`Bool`]: Attr::Bool
    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(..))
    }

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

/// A space-separated series of attributes.
#[derive(Debug, Clone)]
pub struct Attrs(Vec<Attr>);

impl Attrs {
    pub fn new(attrs: Vec<Attr>) -> Self {
        Self(attrs)
    }

    pub fn as_slice(&self) -> &[Attr] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> slice::Iter<'_, Attr> {
        self.0.iter()
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

#[cfg(test)]
mod tests {
    use crate::attribute::{Attrs, BoolAttr};

    use super::KvAttr;

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
        assert_eq!(output.key().repr(), input);
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

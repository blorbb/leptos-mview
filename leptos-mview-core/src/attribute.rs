mod kv;
mod bool;
mod directive;

use core::slice;

use proc_macro2::TokenStream;

use syn::parse::Parse;


use self::{kv::KvAttr, bool::BoolAttr, directive::DirectiveAttr};

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
        } else {
            let bool_attr = input.parse::<BoolAttr>()?;

            Ok(Self::Bool(bool_attr))
        }
    }
}

impl Attr {
    /// Converts an attribute to a `.attr(key, value)` token stream.
    ///
    /// Directives are converted differently, but is compatible with
    /// `leptos::html::*` so will work as expected.
    ///
    /// Other special directives (like `assign`) cannot be used in html
    /// elements, and will cause an abort.
    ///
    /// Some special key properties (like `ref`) are also converted differently.
    pub fn to_attr_method(&self) -> TokenStream {
        match self {
            Self::Kv(attr) => attr.to_attr_method(),
            Self::Bool(attr) => attr.to_attr_method(),
            Self::Directive(attr) => attr.to_attr_method(),
        }
    }

    /// Converts an attribute to a `.key(value)` token stream.
    ///
    /// Only the `on` directive is allowed on components: calling this method
    /// on a `class` or `style` directive will abort.
    pub fn to_component_builder_method(&self) -> TokenStream {
        match self {
            Self::Kv(attr) => attr.to_component_builder_method(),
            Self::Bool(attr) => attr.to_component_builder_method(),
            Self::Directive(attr) => attr.to_component_builder_method(),
        }
    }
}

/// A space-separated series of attributes.
#[derive(Debug, Clone)]
pub struct Attrs(Vec<Attr>);

impl Attrs {
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

        pub fn len(&self) -> usize {
            self.0.len()
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

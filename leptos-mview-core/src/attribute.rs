use core::slice;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{parse::Parse, parse_quote_spanned, Token};

use crate::{error_ext::ResultExt, ident::KebabIdent, kw, value::Value};

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
    value: Value,
}

impl KvAttr {
    pub const fn key(&self) -> &KebabIdent {
        &self.key
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }

    /// Returns a (key, value) tuple.
    pub const fn kv(&self) -> (&KebabIdent, &Value) {
        (self.key(), self.value())
    }
}

impl Parse for KvAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // first check that this is actually a kv attribute.
        // if there is an ident followed by =, this is kv attribute.
        let fork = input.fork();
        fork.parse::<KebabIdent>()?;

        if fork.peek(Token![=]) {
            // this is a kv attribute: consume main input stream.
            let key = input.parse::<KebabIdent>().unwrap();
            input.parse::<Token![=]>().unwrap();
            let value = input.parse::<Value>().unwrap_or_abort();
            Ok(Self { key, value })
        } else {
            Err(input.error("invalid kv attribute"))
        }
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
    pub const fn key(&self) -> &KebabIdent {
        &self.0
    }

    pub fn span(&self) -> proc_macro2::Span {
        self.0.span()
    }

    /// Returns a `true` token spanned to the identifier.
    pub fn spanned_true(&self) -> syn::LitBool {
        parse_quote_spanned!(self.span()=> true)
    }

    /// Returns a key-value pair `(key, true)`, with the `true` being spanned
    /// to the identifier.
    pub fn kv(&self) -> (&KebabIdent, syn::LitBool) {
        (self.key(), self.spanned_true())
    }
}

impl Parse for BoolAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse::<KebabIdent>()?))
    }
}

/// A special directive attribute.
///
/// Currently `class`, `style` and `on` are supported.
///
/// Example:
/// ```text
/// <button class:primary={primary} style:color="grey" on:click={handle_click} />
/// ```
#[derive(Debug, Clone)]
pub enum DirectiveAttr {
    Class {
        class_token: kw::class,
        colon_token: Token![:],
        name: KebabIdent,
        equals_token: Token![=],
        value: Value,
    },
    Style {
        style_token: kw::style,
        colon_token: Token![:],
        name: KebabIdent,
        equals_token: Token![=],
        value: Value,
    },
    On {
        on_token: kw::on,
        colon_token: Token![:],
        event: syn::Ident,
        equals_token: Token![=],
        callback: syn::ExprBlock,
    },
}

impl DirectiveAttr {
    pub fn span(&self) -> Span {
        match self {
            Self::Class {
                class_token, name, ..
            } => class_token
                .span
                .join(name.span())
                .unwrap_or(class_token.span),
            Self::Style {
                style_token, name, ..
            } => style_token
                .span
                .join(name.span())
                .unwrap_or(style_token.span),
            Self::On {
                on_token, event, ..
            } => on_token.span.join(event.span()).unwrap_or(on_token.span),
        }
    }
}

impl Parse for DirectiveAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // attribute should be <kw>:<ident> = <value>
        if !input.peek2(Token![:]) {
            return Err(input.error("invalid directive attribute: colon not found"));
        }

        if input.peek(kw::class) {
            Ok(Self::Class {
                class_token: input.parse().unwrap(),
                colon_token: input.parse().unwrap(),
                name: input
                    .parse()
                    .expect_or_abort("expected class name after `class:` directive"),
                equals_token: input.parse().unwrap_or_abort(),
                value: input.parse().unwrap_or_abort(),
            })
        } else if input.peek(kw::style) {
            Ok(Self::Style {
                style_token: input.parse().unwrap(),
                colon_token: input.parse().unwrap(),
                name: input
                    .parse()
                    .expect_or_abort("expected style name after `style:` directive"),
                equals_token: input.parse().unwrap_or_abort(),
                value: input.parse().unwrap_or_abort(),
            })
        } else if input.peek(kw::on) {
            Ok(Self::On {
                on_token: input.parse().unwrap(),
                colon_token: input.parse().unwrap(),
                event: input
                    .parse()
                    .expect_or_abort("expected event name after `on:` directive"),
                equals_token: input.parse().unwrap_or_abort(),
                callback: input.parse().unwrap_or_abort(),
            })
        } else {
            Err(input.error("unknown directive"))
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
    pub fn to_attr_method(&self) -> TokenStream {
        match self {
            Self::Kv(attr) => {
                let (key, value) = attr.kv();
                quote! {
                    .attr(#key, #[allow(unused_braces)] #value)
                }
            }
            Self::Bool(attr) => {
                let (key, value) = attr.kv();
                quote! {
                    .attr(#key, #value)
                }
            }
            Self::Directive(attr) => match attr {
                DirectiveAttr::Class { name, value, .. } => {
                    quote! {
                        .class(#name, #value)
                    }
                }
                DirectiveAttr::Style { name, value, .. } => {
                    quote! {
                        .style(#name, #value)
                    }
                }
                DirectiveAttr::On {
                    event, callback, ..
                } => {
                    quote! {
                        .on(::leptos::ev::#event, #callback)
                    }
                }
            },
        }
    }

    /// Converts an attribute to a `.key(value)` token stream.
    ///
    /// Only the `on` directive is allowed on components: calling this method
    /// on a `class` or `style` directive will abort.
    pub fn to_component_builder_method(&self) -> TokenStream {
        match self {
            Self::Kv(attr) => {
                let key = attr.key().to_snake_ident();
                let value = attr.value();
                quote! {
                    .#key(#[allow(unused_braces)] #value)
                }
            }
            Self::Bool(attr) => {
                let key = attr.key().to_snake_ident();
                let value = attr.spanned_true();
                quote! {
                    .#key(#value)
                }
            }
            Self::Directive(attr) => match attr {
                DirectiveAttr::On {
                    event, callback, ..
                } => {
                    quote! {
                        .on(
                            ::leptos::ev::undelegated(
                                ::leptos::ev::#event
                            ),
                            #callback
                        )
                    }
                }
                other => abort!(
                    other.span(),
                    "only `on:` directives are allowed on components"
                ),
            },
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

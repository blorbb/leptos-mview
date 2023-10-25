pub mod directive;
pub mod kv;
mod parsing;
pub mod selector;
pub mod spread_attrs;

use std::ops::Deref;

use proc_macro2::Span;
use syn::{ext::IdentExt, parse::Parse, Token};

use self::{directive::DirectiveAttr, kv::KvAttr, spread_attrs::SpreadAttr};
use crate::error_ext::ResultExt;

#[derive(Debug, Clone)]
pub enum Attr {
    Kv(KvAttr),
    Directive(DirectiveAttr),
    Spread(SpreadAttr),
}

impl Attr {
    pub fn span(&self) -> Span {
        match self {
            Self::Kv(kv) => kv.span(),
            Self::Directive(dir) => dir.span(),
            Self::Spread(s) => s.span(),
        }
    }
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // ident then colon must be directive
        // just ident must be regular kv attribute
        // otherwise, try kv or spread
        if input.peek(syn::Ident::peek_any) && input.peek2(Token![:]) {
            let dir = input.parse::<DirectiveAttr>().unwrap_or_abort();
            Ok(Self::Directive(dir))
        } else if input.peek(syn::Ident) {
            let kv = input.parse::<KvAttr>().unwrap_or_abort();
            Ok(Self::Kv(kv))
        } else if let Ok(kv) = input.parse::<KvAttr>() {
            Ok(Self::Kv(kv))
        } else if let Ok(spread) = input.parse::<SpreadAttr>() {
            Ok(Self::Spread(spread))
        } else {
            Err(input.error("no attribute found"))
        }
    }
}

/// A space-separated series of attributes.
#[derive(Debug, Clone)]
pub struct Attrs(Vec<Attr>);

impl Deref for Attrs {
    type Target = [Attr];

    fn deref(&self) -> &Self::Target { &self.0 }
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
    use syn::parse_quote;

    use super::{Attr, KvAttr};
    use crate::attribute::Attrs;

    #[test]
    fn simple_kv_attr() {
        let input: KvAttr = parse_quote! { key = "value" };
        assert_eq!(input.key().repr(), "key");
        assert!(input.value().is_lit());
    }

    #[test]
    fn parse_complex_attrs() {
        impl Attr {
            fn is_kv(&self) -> bool { matches!(self, Self::Kv(..)) }

            fn is_dir(&self) -> bool { matches!(self, Self::Directive(..)) }

            fn is_spread(&self) -> bool { matches!(self, Self::Spread(..)) }
        }
        let attrs: Attrs = parse_quote! {
            key-1 = "value"
            a-long-thing=[some()]
            style:--var-2={move || true}
            class:{disabled}
            {checked}
            {..spread}
        };
        assert!(attrs[0].is_kv());
        assert!(attrs[1].is_kv());
        assert!(attrs[2].is_dir());
        assert!(attrs[3].is_dir());
        assert!(attrs[4].is_kv());
        assert!(attrs[5].is_spread());
    }
}

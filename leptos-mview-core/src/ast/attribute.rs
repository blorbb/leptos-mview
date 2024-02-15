pub mod directive;
pub mod kv;
mod parsing;
pub mod selector;
pub mod spread_attrs;

use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    Token,
};

use self::{directive::DirectiveAttr, kv::KvAttr, spread_attrs::SpreadAttr};
use crate::{error_ext::ResultExt, recover::rollback_err};

#[derive(Clone)]
pub enum Attr {
    Kv(KvAttr),
    Directive(DirectiveAttr),
    Spread(SpreadAttr),
}

// impl Attr {
//     pub fn span(&self) -> Span {
//         match self {
//             Self::Kv(kv) => kv.span(),
//             Self::Directive(dir) => dir.span(),
//             Self::Spread(s) => s.span(),
//         }
//     }
// }

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // ident then colon must be directive
        // just ident must be regular kv attribute
        // otherwise, try kv or spread
        if input.peek(syn::Ident::peek_any) && input.peek2(Token![:]) {
            // cannot be anything else, abort if fails
            let dir = input.parse::<DirectiveAttr>().unwrap_or_abort();
            Ok(Self::Directive(dir))
        } else if input.peek(syn::Ident) {
            // definitely a k-v attribute
            let kv = input.parse::<KvAttr>()?;
            Ok(Self::Kv(kv))
        } else if let Some(kv) = rollback_err(input, KvAttr::parse) {
            // k-v attributes don't necessarily start with ident, try the rest
            Ok(Self::Kv(kv))
        } else if let Some(spread) = rollback_err(input, SpreadAttr::parse) {
            Ok(Self::Spread(spread))
        } else {
            Err(input.error("no attribute found"))
        }
    }
}

/// A space-separated series of attributes.
#[derive(Clone)]
pub struct Attrs(Vec<Attr>);

impl std::ops::Deref for Attrs {
    type Target = [Attr];
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();
        while let Some(inner) = rollback_err(input, Attr::parse) {
            vec.push(inner);
        }
        Ok(Self(vec))
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{Attr, KvAttr};
    use crate::ast::Attrs;

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

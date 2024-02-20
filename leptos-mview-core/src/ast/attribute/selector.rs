use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use crate::{ast::KebabIdent, parse::rollback_err};

/// A shorthand for adding class or ids to an element.
///
/// Classes are added with a preceding `.`, ids with a `#`.
///
/// # Example
/// ```ignore
/// div.a-class.small.big.big-big;
/// ```
///
/// The `#` before the id needs a space before it due to
/// [Reserving syntax](https://doc.rust-lang.org/edition-guide/rust-2021/reserving-syntax.html)
/// since Rust 2021.
/// ```ignore
/// div #important .more-classes #another-id .claaass
/// ```
#[derive(Clone)]
pub enum SelectorShorthand {
    Id {
        pound_symbol: Token![#],
        id: KebabIdent,
    },
    Class {
        dot_symbol: Token![.],
        class: KebabIdent,
    },
}

impl SelectorShorthand {
    pub const fn ident(&self) -> &KebabIdent {
        match self {
            Self::Id { id, .. } => id,
            Self::Class { class, .. } => class,
        }
    }

    pub fn prefix(&self) -> proc_macro2::Punct {
        let (char, span) = match self {
            Self::Id { pound_symbol, .. } => ('#', pound_symbol.span),
            Self::Class { dot_symbol, .. } => ('.', dot_symbol.span),
        };
        let mut punct = proc_macro2::Punct::new(char, proc_macro2::Spacing::Alone);
        punct.set_span(span);
        punct
    }

    // pub fn span(&self) -> Span { span::join(self.prefix().span(),
    // self.ident().span()) }
}

impl Parse for SelectorShorthand {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Some(dot) = rollback_err(input, <Token![.]>::parse) {
            let class = input.parse::<KebabIdent>()?;
            Ok(Self::Class {
                dot_symbol: dot,
                class,
            })
        } else if let Some(pound) = rollback_err(input, <Token![#]>::parse) {
            let id = input.parse::<KebabIdent>()?;
            Ok(Self::Id {
                pound_symbol: pound,
                id,
            })
        } else {
            Err(input.error("no class or id shorthand found"))
        }
    }
}

#[derive(Clone, Default)]
pub struct SelectorShorthands(Vec<SelectorShorthand>);

impl std::ops::Deref for SelectorShorthands {
    type Target = [SelectorShorthand];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl Parse for SelectorShorthands {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();
        while let Some(inner) = rollback_err(input, SelectorShorthand::parse) {
            vec.push(inner);
        }

        Ok(Self(vec))
    }
}

#[cfg(test)]
mod tests {
    use super::{SelectorShorthand, SelectorShorthands};

    #[derive(PartialEq, Eq)]
    enum SelectorKind {
        Class,
        Id,
    }

    #[test]
    fn multiple() {
        let stream = ".class.another-class #id #id2 .wow-class #ida";
        let selectors: SelectorShorthands = syn::parse_str(stream).unwrap();
        let result = [
            (SelectorKind::Class, "class"),
            (SelectorKind::Class, "another-class"),
            (SelectorKind::Id, "id"),
            (SelectorKind::Id, "id2"),
            (SelectorKind::Class, "wow-class"),
            (SelectorKind::Id, "ida"),
        ]
        .into_iter();
        for (selector, result) in selectors.iter().zip(result) {
            match selector {
                SelectorShorthand::Id { id, .. } => {
                    assert!(
                        result.0 == SelectorKind::Id,
                        "{} should not be an id",
                        id.repr()
                    );
                    assert_eq!(result.1, id.repr());
                }
                SelectorShorthand::Class { class, .. } => {
                    assert!(
                        result.0 == SelectorKind::Class,
                        "{} should not be a class",
                        class.repr()
                    );
                    assert_eq!(result.1, class.repr());
                }
            }
        }
    }
}

use proc_macro2::Span;
use syn::{parse::Parse, Token};

use super::derive_multi_ast_for;
use crate::{ast::KebabIdent, error_ext::ResultExt, span};

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
#[derive(Debug, Clone)]
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

    pub fn span(&self) -> Span { span::join(self.prefix().span(), self.ident().span()) }
}

impl Parse for SelectorShorthand {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(dot) = input.parse::<Token![.]>() {
            let class = input.parse::<KebabIdent>().unwrap_or_abort();
            Ok(Self::Class {
                dot_symbol: dot,
                class,
            })
        } else if let Ok(pound) = input.parse::<Token![#]>() {
            let id = input.parse::<KebabIdent>().unwrap_or_abort();
            Ok(Self::Id {
                pound_symbol: pound,
                id,
            })
        } else {
            Err(input.error("no class or id shorthand found"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectorShorthands(Vec<SelectorShorthand>);

derive_multi_ast_for! {
    struct SelectorShorthands(Vec<SelectorShorthand>);
    impl Parse(allow_non_empty);
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

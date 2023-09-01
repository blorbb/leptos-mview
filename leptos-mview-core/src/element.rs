use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_quote_spanned, Token};

use crate::{
    attribute::{Attr, Attrs},
    children::Children,
    ident::KebabIdent,
    value::Value,
};

#[derive(Debug, PartialEq, Eq)]
pub enum TagKind {
    Html,
    Component,
    Svg,
    Unknown,
}

impl From<&str> for TagKind {
    fn from(value: &str) -> Self {
        if is_component(value) {
            Self::Component
        } else if is_svg_element(value) {
            Self::Svg
        } else if is_unknown_element(value) {
            Self::Unknown
        } else {
            Self::Html
        }
    }
}

#[derive(Debug)]
pub enum Tag {
    Html(syn::Ident),
    Component(syn::Ident),
    Svg(syn::Ident),
    Unknown(KebabIdent),
}

impl Tag {
    /// Returns `true` if the tag is [`Html`].
    ///
    /// [`Html`]: Tag::Html
    #[must_use]
    pub fn is_html(&self) -> bool {
        matches!(self, Self::Html(..))
    }

    /// Returns `true` if the tag is [`Component`].
    ///
    /// [`Component`]: Tag::Component
    #[must_use]
    pub fn is_component(&self) -> bool {
        matches!(self, Self::Component(..))
    }

    /// Returns `true` if the tag is [`Svg`].
    ///
    /// [`Svg`]: Tag::Svg
    #[must_use]
    pub fn is_svg(&self) -> bool {
        matches!(self, Self::Svg(..))
    }

    /// Returns `true` if the tag is [`Unknown`].
    ///
    /// [`Unknown`]: Tag::Unknown
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(..))
    }
}

impl Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<KebabIdent>()?;
        let kind = TagKind::from(ident.repr());
        Ok(match kind {
            TagKind::Html => Self::Html(ident.to_snake_ident()),
            TagKind::Component => Self::Component(ident.to_snake_ident()),
            TagKind::Svg => Self::Svg(ident.to_snake_ident()),
            TagKind::Unknown => Self::Unknown(ident),
        })
    }
}

/// A HTML or custom component.
///
/// Syntax mostly looks like this:
/// ```text
/// div class="blue" { "hello!" }
/// ^^^ ^^^^^^^^^^^^   ^^^^^^^^
/// tag attributes     children
/// ```
///
/// If the element ends in a semicolon, `children` is `None`.
/// ```text
/// input type="text";
/// br;
/// ```
#[derive(Debug)]
pub struct Element {
    tag: Tag,
    attrs: Attrs,
    children: Option<Children>,
}

impl Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag: Tag = input.parse()?;
        let attrs: Attrs = input.parse()?;

        if input.peek(Token![;]) {
            // no children, terminated by semicolon.
            input.parse::<Token![;]>().unwrap();
            Ok(Self::new(tag, attrs, None))
        } else if input.peek(syn::token::Brace) {
            // has children in brace.
            let children;
            syn::braced!(children in input);
            let children: Children = children.parse()?;
            Ok(Self::new(tag, attrs, Some(children)))
        } else {
            abort!(
                input.span(), "child elements not found";
                note = "if you don't want any child elements, end the element with \
                a semi-colon `;` or empty braces `{}`."
            )
        }
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // HTML only for now
        match self.tag() {
            Tag::Html(ident) => {
                tokens.extend(quote! {
                    ::leptos::html::#ident()
                });
            }
            Tag::Component(_) => {
                let stream = self.to_component_token_stream().unwrap();
                tokens.extend(stream);
                return;
            }
            Tag::Svg(..) => todo!(),
            Tag::Unknown(..) => todo!(),
        };

        for attr in self.attrs().iter() {
            match attr {
                Attr::Kv(kv) => {
                    let ident = kv.key();
                    let value = kv.value();
                    tokens.extend(quote! {
                        .attr(#ident, #value)
                    })
                }
                Attr::Bool(b) => {
                    let ident = b.key();
                    tokens.extend(quote! {
                        .attr(#ident, true)
                    });
                }
            }
        }

        if let Some(children) = self.children() {
            tokens.extend(children.to_child_methods())
        }
    }
}

impl Element {
    pub fn new(tag: Tag, attrs: Attrs, children: Option<Children>) -> Self {
        Self {
            tag,
            attrs,
            children,
        }
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }

    pub fn children(&self) -> Option<&Children> {
        self.children.as_ref()
    }

    /// Transforms a component into a `TokenStream` of a leptos component view.
    ///
    /// Returns `None` if `self.tag` is not a `Component`.
    ///
    /// Example builder expansion of a component:

    /// ```ignore
    /// leptos::component_view(
    ///     &Com,
    ///     leptos::component_props_builder(&Com)
    ///         .num(3)
    ///         .text("a".to_string())
    ///         .children(Box::new(move || {
    ///             Fragment::lazy(|| [
    ///                 "child",
    ///                 "child2",
    ///             ])
    ///         }))
    ///         .build()
    /// )
    /// ```
    ///
    /// Where the component has signature:
    ///
    /// ```ignore
    /// #[component]
    /// pub fn Com(num: u32, text: String, children: Children) -> impl IntoView { ... }
    /// ```
    fn to_component_token_stream(&self) -> Option<TokenStream> {
        let Tag::Component(ident) = self.tag() else {
            return None;
        };
        let keys = self.attrs().iter().map(|attr| match attr {
            Attr::Kv(kv) => kv.key(),
            Attr::Bool(b) => b.key(),
        });
        let values = self.attrs().iter().map(|attr| match attr {
            Attr::Kv(kv) => kv.value().clone(),
            Attr::Bool(b) => Value::Lit(parse_quote_spanned!(b.span()=> true)),
        });
        // .children takes a boxed fragment
        let children_fragment = self.children().map(|children| children.to_fragment());
        let children = if let Some(tokens) = children_fragment {
            quote! {
                .children(
                    ::std::boxed::Box::new(move || #tokens)
                )
            }
        } else {
            quote! {}
        };

        Some(quote! {
            ::leptos::component_view(
                &#ident,
                ::leptos::component_props_builder(&Comp)
                    #( .#keys(#values) )*
                    #children
                    .build()
            )
        })
    }
}

pub fn is_component(tag: &str) -> bool {
    tag.starts_with(|c: char| c.is_ascii_uppercase())
}

pub fn is_svg_element(tag: &str) -> bool {
    [
        "animate",
        "animateMotion",
        "animateTransform",
        "circle",
        "clipPath",
        "defs",
        "desc",
        "discard",
        "ellipse",
        "feBlend",
        "feColorMatrix",
        "feComponentTransfer",
        "feComposite",
        "feConvolveMatrix",
        "feDiffuseLighting",
        "feDisplacementMap",
        "feDistantLight",
        "feDropShadow",
        "feFlood",
        "feFuncA",
        "feFuncB",
        "feFuncG",
        "feFuncR",
        "feGaussianBlur",
        "feImage",
        "feMerge",
        "feMergeNode",
        "feMorphology",
        "feOffset",
        "fePointLight",
        "feSpecularLighting",
        "feSpotLight",
        "feTile",
        "feTurbulence",
        "filter",
        "foreignObject",
        "g",
        "hatch",
        "hatchpath",
        "image",
        "line",
        "linearGradient",
        "marker",
        "mask",
        "metadata",
        "mpath",
        "path",
        "pattern",
        "polygon",
        "polyline",
        "radialGradient",
        "rect",
        "set",
        "stop",
        "svg",
        "switch",
        "symbol",
        "text",
        "textPath",
        "tspan",
        "use",
        "use_",
        "view",
    ]
    .binary_search(&tag)
    .is_ok()
}

pub fn is_unknown_element(tag: &str) -> bool {
    // web components are required to have a dash
    tag.contains('-')
}

#[cfg(test)]
mod tests {
    use super::Element;

    #[test]
    fn full_element() {
        let input = r#"div class="test" checked data-index=(index) { "child" span { "child2" } }"#;
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 3);
        assert!(element.children().is_some());
        assert_eq!(element.children().unwrap().len(), 2);
    }

    #[test]
    fn no_child_element() {
        let input = r#"input type="text";"#;
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 1);
        assert!(element.children().is_none());
    }

    #[test]
    fn no_child_or_attrs() {
        let input = "br;";
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 0);
        assert!(element.children.is_none());
    }
}

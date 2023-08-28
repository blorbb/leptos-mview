use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Token};

use crate::{attribute::{Attrs, Attr}, children::Children, ident::KebabIdent};

#[derive(Debug, PartialEq, Eq)]
pub enum TagKind {
    Html,
    Svg,
    Unknown,
    Component,
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
pub struct Tag {
    kind: TagKind,
    ident: KebabIdent,
}

impl Tag {
    pub fn is_html(&self) -> bool {
        self.kind == TagKind::Html
    }
    pub fn is_svg(&self) -> bool {
        self.kind == TagKind::Svg
    }
    pub fn is_component(&self) -> bool {
        self.kind == TagKind::Component
    }
    pub fn is_unknown(&self) -> bool {
        self.kind == TagKind::Unknown
    }

    pub fn ident(&self) -> &KebabIdent {
        &self.ident
    }

    pub fn kind(&self) -> &TagKind {
        &self.kind
    }
}

impl Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<KebabIdent>()?;
        let kind = TagKind::from(ident.repr());
        Ok(Self { ident, kind })
    }
}

/// A HTML or custom component.
///
/// Syntax mostly looks like this:
/// ```text
/// div class="blue" { "hello!" }
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
        match self.tag().kind() {
            TagKind::Html => {
                let ident = self.tag().ident().to_snake_ident();
                tokens.extend(quote! {
                    ::leptos::html::#ident()
                });
            }
            TagKind::Svg => todo!(),
            TagKind::Unknown => todo!(),
            TagKind::Component => todo!(),
        };

        for attr in self.attrs().attrs() {
            match attr {
                Attr::Kv(kv) => {
                    let ident = kv.key();
                    let value = kv.value();
                    tokens.extend(quote! {
                        .attr(#ident, #value)
                    })
                },
                Attr::Bool(b) => {
                    let ident = b.key();
                    tokens.extend(quote! {
                        .attr(#ident, true)
                    });
                },
            }
        }

        if let Some(children) = self.children() {
            for child in children.children() {
                tokens.extend(quote! {
                    .child(#child)
                });
            }
        }
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

use proc_macro2::Span;
use syn::parse::Parse;

use crate::ident::KebabIdent;

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
    pub fn span(&self) -> Span {
        match self {
            Self::Html(ident) | Self::Component(ident) | Self::Svg(ident) => ident.span(),
            Self::Unknown(ident) => ident.span(),
        }
    }

    pub const fn kind(&self) -> TagKind {
        match self {
            Self::Html(_) => TagKind::Html,
            Self::Component(_) => TagKind::Component,
            Self::Svg(_) => TagKind::Svg,
            Self::Unknown(_) => TagKind::Unknown,
        }
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

use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{parse::Parse, parse_quote, Token};

use crate::{error_ext::ResultExt, ident::KebabIdent};

#[derive(Debug, PartialEq, Eq)]
pub enum TagKind {
    Html,
    Component,
    Svg,
    Math,
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
        } else if is_math_ml_element(value) {
            Self::Math
        } else {
            Self::Html
        }
    }
}

#[derive(Debug)]
pub enum Tag {
    Html(syn::Ident),
    /// The generic will contain a leading `::`.
    Component(syn::Ident, Option<syn::AngleBracketedGenericArguments>),
    Svg(syn::Ident),
    Math(syn::Ident),
    Unknown(KebabIdent),
}

impl Tag {
    pub fn span(&self) -> Span {
        match self {
            Self::Html(ident)
            | Self::Component(ident, _)
            | Self::Svg(ident)
            | Self::Math(ident) => ident.span(),
            Self::Unknown(ident) => ident.span(),
        }
    }

    pub const fn kind(&self) -> TagKind {
        match self {
            Self::Html(_) => TagKind::Html,
            Self::Component(..) => TagKind::Component,
            Self::Svg(_) => TagKind::Svg,
            Self::Math(_) => TagKind::Math,
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
            TagKind::Component => {
                if input.peek(Token![::]) {
                    abort! {
                        input.span(), "unexpected token `::`";
                        help = "turbofish syntax is not used for component generics, \
                        place angle brackets directly after the component name"
                    }
                }
                let generics = input.peek(Token![<]).then(|| {
                    let non_leading_generic = input
                        .parse::<syn::AngleBracketedGenericArguments>()
                        .expect_or_abort("failed to parse component generics");
                    parse_quote!(::#non_leading_generic)
                });
                Self::Component(ident.to_snake_ident(), generics)
            }
            TagKind::Svg => Self::Svg(ident.to_snake_ident()),
            TagKind::Math => Self::Math(ident.to_snake_ident()),
            TagKind::Unknown => Self::Unknown(ident),
        })
    }
}

#[rustfmt::skip]
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

fn is_math_ml_element(tag: &str) -> bool {
    [
        "annotation",
        "maction",
        "math",
        "menclose",
        "merror",
        "mfenced",
        "mfrac",
        "mi",
        "mmultiscripts",
        "mn",
        "mo",
        "mover",
        "mpadded",
        "mphantom",
        "mprescripts",
        "mroot",
        "mrow",
        "ms",
        "mspace",
        "msqrt",
        "mstyle",
        "msub",
        "msubsup",
        "msup",
        "mtable",
        "mtd",
        "mtext",
        "mtr",
        "munder",
        "munderover",
        "semantics",
    ]
    .binary_search(&tag)
    .is_ok()
}

pub fn is_unknown_element(tag: &str) -> bool {
    // web components are required to have a dash
    tag.contains('-')
}

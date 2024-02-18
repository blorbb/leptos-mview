use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Token,
};

use crate::ast::KebabIdent;

#[allow(clippy::doc_markdown)]
/// The name of the element, like `div`, `path`, `For`, `leptos-island`, etc.
///
/// All tags except web-components are parsed as a [`syn::Ident`].
/// Whether elements are an HTML, SVG or MathML tag is based on a list: SVG and
/// MathML are searched for first, everything else is considered to be an HTML
/// element.
///
/// All web-components have a `-` in them, so they are parsed as a
/// [`KebabIdent`].
///
/// All leptos components are in `UpperCamelCase`, so any tags
/// that start with a capital letter are considered components. Generics are
/// supported and stored in this enum, if there are any after a leptos
/// component. Turbofish syntax (`Component::<T>`) is not used, the generic is
/// placed directly after (`Component<T>`).
///
/// See [`TagKind`] for a discriminant-only version of this enum.
///
/// # Parsing
/// If parsing of a [`KebabIdent`] fails, an [`Err`] will be returned and the
/// [`ParseStream`] will not be advanced. However, if a [`Tag::Component`] is
/// found and there are generics, parsing will **abort** if parsing the generics
/// fails.
pub enum Tag {
    Html(syn::Ident),
    /// The generic will contain a leading `::`.
    Component(syn::Path),
    Svg(syn::Ident),
    Math(syn::Ident),
    WebComponent(KebabIdent),
}

impl Tag {
    /// Returns the [`Span`] of the tag identifier.
    ///
    /// Component generics are not included in this span.
    ///
    /// Use the [`Tag::ident`] function if the identifier itself is required.
    pub fn span(&self) -> Span {
        match self {
            Self::Html(ident) | Self::Svg(ident) | Self::Math(ident) => ident.span(),
            Self::WebComponent(ident) => ident.span(),
            Self::Component(path) => path.span(),
        }
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // peek 1 in case it's a leading ::
        // this will also include any generics
        // also look for generics without a full path
        if input.peek2(Token![::]) || input.peek(Token![::]) || input.peek2(Token![<]) {
            // this is a path segment: must be a component
            let path = syn::Path::parse(input)?;
            return Ok(Self::Component(path));
        }

        let ident = input.parse::<KebabIdent>()?;
        let kind = TagKind::from(ident.repr());
        Ok(match kind {
            TagKind::Html => Self::Html(ident.to_snake_ident()),
            TagKind::Component => Self::Component(syn::Path::from(ident.to_snake_ident())),
            TagKind::Svg => Self::Svg(ident.to_snake_ident()),
            TagKind::Math => Self::Math(ident.to_snake_ident()),
            TagKind::WebComponent => Self::WebComponent(ident),
        })
    }
}

/// Discriminant-only enum for [`Tag`].
#[derive(Debug, PartialEq, Eq)]
pub enum TagKind {
    Html,
    Component,
    Svg,
    Math,
    WebComponent,
}

impl From<&str> for TagKind {
    /// Figures out the kind of element the provided tag is.
    ///
    /// The [`&str`](str) passed in should be a valid tag identifier, i.e. a
    /// valid Rust ident or [`KebabIdent`].
    fn from(value: &str) -> Self {
        if is_component(value) {
            Self::Component
        } else if is_svg_element(value) {
            Self::Svg
        } else if is_web_component(value) {
            Self::WebComponent
        } else if is_math_ml_element(value) {
            Self::Math
        } else {
            Self::Html
        }
    }
}

/// Whether the tag is a leptos component.
///
/// Checks if the first character is uppercase.
///
/// The [`&str`](str) passed in should be a valid tag identifier, i.e. a
/// valid Rust ident or [`KebabIdent`].
#[rustfmt::skip]
pub fn is_component(tag: &str) -> bool {
    tag.starts_with(|c: char| c.is_ascii_uppercase())
}

/// Whether the tag is an SVG element.
///
/// Checks based on a list.
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

/// Whether the tag is an SVG element.
///
/// Checks based on a list.
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

/// Whether the tag is a web-component.
///
/// The [`&str`](str) passed in should be a valid tag identifier, i.e. a
/// valid Rust ident or [`KebabIdent`].
///
/// Returns `true` if the tag contains a `-` as all web-components require a
/// `-`.
pub fn is_web_component(tag: &str) -> bool { tag.contains('-') }

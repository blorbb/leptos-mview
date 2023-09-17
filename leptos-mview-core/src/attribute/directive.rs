use core::fmt;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream},
    Token,
};

use crate::{error_ext::ResultExt, ident::KebabIdent, kw, value::Value};

use super::{
    parsing::{parse_braced_bool, parse_dir_then, parse_ident_braced, parse_str_braced},
    ShorthandAttr,
};

/// A special attribute like `on:click={...}`.
///
/// # Examples
/// ```ignore
/// input type="checkbox" on:input={handle_input};
///                       ^^^^^^^^^^^^^^^^^^^^^^^
/// button class:primary={primary} style:color="grey";
///        ^^^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^
/// ```
/// The shorthand syntax is also supported on the argument of directives:
/// ```ignore
/// button class:{primary} style:color="grey";
/// ```
///
/// # Parsing
/// Parsing will fail if no `:` is found. The `ParseStream` will not be
/// advnaced in this case.
///
/// If a `:` is found but any other part of the parsing fails (including unknown
/// directives), the macro will abort.
#[derive(Debug, Clone)]
pub struct DirectiveAttr {
    directive: DirectiveIdent,
    name: KebabIdent,
    value: Value,
}

impl DirectiveAttr {
    /// Returns the part before the equal sign on nightly.
    ///
    /// If compiled on stable, the span will only be the directive (e.g. `on`).
    ///
    /// Example on nightly:
    /// ```ignore
    /// button on:click={handle_click}
    ///        ^^^^^^^^
    /// ```
    pub fn span(&self) -> Span {
        self.directive()
            .span()
            .join(self.name.span())
            .unwrap_or(self.directive().span())
    }

    pub const fn directive(&self) -> &DirectiveIdent {
        &self.directive
    }

    pub const fn name(&self) -> &KebabIdent {
        &self.name
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }

    pub const fn kind(&self) -> &DirectiveKind {
        self.directive().kind()
    }
}

impl Parse for DirectiveAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // attribute should be <dir>:<name> = <value>
        if !input.peek2(Token![:]) {
            return Err(input.error("invalid directive attribute: colon not found"));
        }
        // after this, any failure to parse should abort.

        let directive = input.parse::<DirectiveIdent>().unwrap_or_abort();
        input.parse::<Token![:]>().unwrap();

        // either a shorthand if there are braces, or the full expression.
        if input.peek(syn::token::Brace) {
            let attr = input.parse::<ShorthandAttr>().unwrap_or_abort();
            Ok(Self {
                directive,
                name: attr.key,
                value: attr.value,
            })
        } else {
            let name = input
                .parse::<KebabIdent>()
                .expect_or_abort_with_msg(&format!(
                    "expected identifier after `{}:` directive",
                    directive.ident()
                ));
            input.parse::<Token![=]>().unwrap_or_abort();
            let value = input.parse::<Value>().unwrap_or_abort();

            Ok(Self {
                directive,
                name,
                value,
            })
        }
    }
}

/// A `class:some-thing={move || boolean()}` directive.
pub struct Class {
    directive: kw::class,
    class_name: syn::LitStr,
    value: Value,
}

impl Parse for Class {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (class, (name, value)) = parse_dir_then(input, parse_str_braced)?;
        Ok(Self {
            directive: class,
            class_name: name,
            value,
        })
    }
}

pub struct Style {
    directive: kw::style,
    style_name: syn::LitStr,
    value: Value,
}

impl Parse for Style {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (style, (name, value)) = parse_dir_then(input, parse_str_braced)?;
        Ok(Self {
            directive: style,
            style_name: name,
            value,
        })
    }
}

pub struct Attr {
    directive: kw::attr,
    attr_key: KebabIdent,
    value: Value,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (attr, (key, value)) = parse_dir_then(input, parse_braced_bool)?;
        Ok(Self {
            directive: attr,
            attr_key: key,
            value,
        })
    }
}

pub struct On {
    directive: kw::on,
    event: syn::Ident,
    value: Value,
}

impl Parse for On {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (on, (event, value)) = parse_dir_then(input, parse_ident_braced)?;
        Ok(Self {
            directive: on,
            event,
            value,
        })
    }
}

pub struct Prop {
    directive: kw::prop,
    name: syn::Ident,
    value: Value,
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (prop, (name, value)) = parse_dir_then(input, parse_ident_braced)?;
        Ok(Self {
            directive: prop,
            name,
            value,
        })
    }
}

pub struct Clone {
    directive: kw::clone,
    name: syn::Ident,
    value: Value,
}

impl Parse for Clone {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (clone, (name, value)) = parse_dir_then(input, parse_ident_braced)?;
        Ok(Self {
            directive: clone,
            name,
            value,
        })
    }
}

/// Holds the identifier for a valid directive.
///
/// # Parsing
/// The `parse` method looks for an ident and validates it. An `Err` is
/// returned if it does not find an ident or if the identifier is not a valid
/// directive.
#[derive(Debug, Clone)]
pub struct DirectiveIdent {
    kind: DirectiveKind,
    ident: syn::Ident,
}

impl DirectiveIdent {
    pub const fn kind(&self) -> &DirectiveKind {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.ident.span()
    }

    pub const fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}

impl Parse for DirectiveIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        if let Ok(ident) = fork.call(syn::Ident::parse_any) {
            let kind = match ident.to_string().as_str() {
                "class" => DirectiveKind::Class,
                "style" => DirectiveKind::Style,
                "on" => DirectiveKind::On,
                "clone" => DirectiveKind::Clone,
                "prop" => DirectiveKind::Prop,
                "attr" => DirectiveKind::Attr,
                _ => return Err(input.error(format!("unknown directive `{ident}`"))),
            };
            // only move input forward if it worked
            input.advance_to(&fork);
            Ok(Self { kind, ident })
        } else {
            Err(input.error("expected identifier"))
        }
    }
}

impl ToTokens for DirectiveIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.ident().to_token_stream());
    }
}

impl fmt::Display for DirectiveIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ident().fmt(f)
    }
}

/// The kinds of supported directives.
#[derive(Debug, Clone)]
pub enum DirectiveKind {
    Style,
    Class,
    On,
    Clone,
    Prop,
    Attr,
}

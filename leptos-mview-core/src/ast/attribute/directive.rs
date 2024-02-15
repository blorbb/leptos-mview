use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    token::Token,
    Token,
};

use super::parsing::{
    parse_dir_then, parse_ident_optional_value, parse_ident_or_braced,
    parse_kebab_or_braced_or_bool, parse_kebab_or_braced_or_str,
};
use crate::{
    ast::{KebabIdent, Value},
    kw,
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
#[derive(Clone)]
pub enum DirectiveAttr {
    Class(Class),
    Style(Style),
    Attr(Attr),
    On(On),
    Prop(Prop),
    Clone(Clone),
    Use(Use),
}

// impl DirectiveAttr {
//     pub fn span(&self) -> Span {
//         match self {
//             Self::Class(a) => a.full_span(),
//             Self::Style(a) => a.full_span(),
//             Self::Attr(a) => a.full_span(),
//             Self::On(a) => a.full_span(),
//             Self::Prop(a) => a.full_span(),
//             Self::Clone(a) => a.full_span(),
//             Self::Use(a) => a.full_span(),
//         }
//     }
// }

impl Parse for DirectiveAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::class) {
            Ok(Self::Class(Class::parse(input)?))
        } else if input.peek(kw::style) {
            Ok(Self::Style(Style::parse(input)?))
        } else if input.peek(kw::attr) {
            Ok(Self::Attr(Attr::parse(input)?))
        } else if input.peek(kw::on) {
            Ok(Self::On(On::parse(input)?))
        } else if input.peek(kw::prop) {
            Ok(Self::Prop(Prop::parse(input)?))
        } else if input.peek(kw::clone) {
            Ok(Self::Clone(Clone::parse(input)?))
        } else if input.peek(Token![use]) {
            Ok(Self::Use(Use::parse(input)?))
        } else {
            Err(input.error("unknown directive"))
        }
    }
}

macro_rules! create_directive {
    ($struct_name:ident { $dir:ty : $key:ty = $value:ty } uses $parser:ident) => {
        #[derive(Clone)]
        pub struct $struct_name {
            dir: $dir,
            key: $key,
            value: $value,
        }

        impl Parse for $struct_name {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let (dir, (key, value)) = parse_dir_then(input, $parser)?;
                Ok(Self { dir, key, value })
            }
        }

        #[allow(dead_code)]
        impl $struct_name {
            pub fn dir_name() -> &'static str { <$dir>::display() }

            pub const fn dir(&self) -> &$dir { &self.dir }
            pub const fn key(&self) -> &$key { &self.key }
            pub const fn value(&self) -> &$value { &self.value }

            pub const fn explode(&self) -> (&$dir, &$key, &$value) {
                (self.dir(), self.key(), self.value())
            }

            pub fn full_span(&self) -> Span {
                crate::span::join(self.dir().span, self.key().span())
            }
        }
    };
    // no value
    ($struct_name:ident { $dir:ty : $key:ty } uses $parser:expr) => {
        #[derive(Clone)]
        pub struct $struct_name {
            dir: $dir,
            key: $key,
        }

        impl Parse for $struct_name {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let (dir, key) = parse_dir_then(input, $parser)?;
                Ok(Self { dir, key })
            }
        }

        #[allow(dead_code)]
        impl $struct_name {
            pub fn dir_name() -> &'static str { <$dir>::display() }

            pub const fn dir(&self) -> &$dir { &self.dir }
            pub const fn key(&self) -> &$key { &self.key }

            pub const fn explode(&self) -> (&$dir, &$key) { (self.dir(), self.key()) }

            pub fn full_span(&self) -> Span {
                crate::span::join(self.dir().span, self.key().span())
            }
        }
    };
}

create_directive! { Class { kw::class : syn::LitStr = Value } uses parse_kebab_or_braced_or_str }
create_directive! { Style { kw::style : syn::LitStr = Value } uses parse_kebab_or_braced_or_str }
create_directive! { Attr { kw::attr : KebabIdent = Value } uses parse_kebab_or_braced_or_bool }
create_directive! { On { kw::on : syn::Ident = Value } uses parse_ident_or_braced }
create_directive! { Prop { kw::prop : syn::Ident = Value } uses parse_ident_or_braced }
create_directive! { Clone { kw::clone : syn::Ident } uses syn::Ident::parse }
create_directive! { Use { Token![use] : syn::Ident = Option<Value> } uses parse_ident_optional_value }

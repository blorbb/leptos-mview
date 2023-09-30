use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

use super::parsing::{
    parse_dir_then, parse_ident_or_braced, parse_kebab_or_braced_or_bool,
    parse_kebab_or_braced_or_str,
};
use crate::{ident::KebabIdent, kw, value::Value};

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
pub enum DirectiveAttr {
    Class(Class),
    Style(Style),
    Attr(Attr),
    On(On),
    Prop(Prop),
    Clone(Clone),
}

impl DirectiveAttr {
    pub fn span(&self) -> Span {
        match self {
            Self::Class(a) => a.full_span(),
            Self::Style(a) => a.full_span(),
            Self::Attr(a) => a.full_span(),
            Self::On(a) => a.full_span(),
            Self::Prop(a) => a.full_span(),
            Self::Clone(a) => a.full_span(),
        }
    }
}

impl Parse for DirectiveAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(class) = input.parse::<Class>() {
            Ok(Self::Class(class))
        } else if let Ok(style) = input.parse::<Style>() {
            Ok(Self::Style(style))
        } else if let Ok(attr) = input.parse::<Attr>() {
            Ok(Self::Attr(attr))
        } else if let Ok(on) = input.parse::<On>() {
            Ok(Self::On(on))
        } else if let Ok(prop) = input.parse::<Prop>() {
            Ok(Self::Prop(prop))
        } else if let Ok(clone) = input.parse::<Clone>() {
            Ok(Self::Clone(clone))
        } else {
            Err(input.error("unknown directive"))
        }
    }
}

pub trait Directive {
    type Dir: ToTokens + syn::token::CustomToken + std::clone::Clone;
    type Key: ToTokens + std::clone::Clone;
    type Value: ToTokens + std::clone::Clone;
    fn dir(&self) -> &Self::Dir;
    fn key(&self) -> &Self::Key;
    fn value(&self) -> &Self::Value;

    fn explode(&self) -> (&Self::Dir, &Self::Key, &Self::Value) {
        (self.dir(), self.key(), self.value())
    }

    fn full_span(&self) -> Span;
}

macro_rules! create_directive {
    ($struct_name:ident { $dir:ty : $key:ty = $value:ty } uses $parser:ident) => {
        #[derive(Debug, Clone)]
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

        impl Directive for $struct_name {
            type Dir = $dir;
            type Key = $key;
            type Value = $value;

            fn value(&self) -> &Self::Value { &self.value }

            fn key(&self) -> &Self::Key { &self.key }

            fn dir(&self) -> &Self::Dir { &self.dir }

            fn full_span(&self) -> Span { crate::span::join(self.dir().span, self.value().span()) }
        }
    };
}

create_directive! { Class { kw::class : syn::LitStr = Value } uses parse_kebab_or_braced_or_str }
create_directive! { Style { kw::style : syn::LitStr = Value } uses parse_kebab_or_braced_or_str }
create_directive! { Attr { kw::attr : KebabIdent = Value } uses parse_kebab_or_braced_or_bool }
create_directive! { On { kw::on : syn::Ident = Value } uses parse_ident_or_braced }
create_directive! { Prop { kw::prop : syn::Ident = Value } uses parse_ident_or_braced }
create_directive! { Clone { kw::clone : syn::Ident = Value } uses parse_ident_or_braced }

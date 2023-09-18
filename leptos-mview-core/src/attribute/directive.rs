use syn::parse::{Parse, ParseStream};

use super::parsing::{parse_braced_bool, parse_dir_then, parse_ident_braced, parse_str_braced};
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
            Err(input.error("invalid directive"))
        }
    }
}

macro_rules! create_directive {
    (
        use $parser:ident for pub struct $struct_name:ident {
            $dir:ident: $dir_type:ty,
            $name:ident: $name_type:ty,
            $value:ident: $value_type:ty,
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            $dir: $dir_type,
            $name: $name_type,
            $value: $value_type,
        }

        impl Parse for $struct_name {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let ($dir, ($name, $value)) = parse_dir_then(input, $parser)?;
                Ok(Self {
                    $dir,
                    $name,
                    $value,
                })
            }
        }

        impl $struct_name {
            pub fn $value(&self) -> &$value_type {
                &self.$value
            }

            pub fn $name(&self) -> &$name_type {
                &self.$name
            }

            pub fn $dir(&self) -> &$dir_type {
                &self.$dir
            }

            pub fn explode(&self) -> (&$dir_type, &$name_type, &$value_type) {
                (&self.$dir(), &self.$name(), &self.$value())
            }
        }
    };
}

create_directive! {
    use parse_str_braced for pub struct Class {
        directive: kw::class,
        class_name: syn::LitStr,
        value: Value,
    }
}

create_directive! {
    use parse_str_braced for pub struct Style {
        directive: kw::style,
        style: syn::LitStr,
        value: Value,
    }
}

create_directive! {
    use parse_braced_bool for pub struct Attr {
        directive: kw::attr,
        key: KebabIdent,
        value: Value,
    }
}

create_directive! {
    use parse_ident_braced for pub struct On {
        directive: kw::on,
        event: syn::Ident,
        value: Value,
    }
}

create_directive! {
    use parse_ident_braced for pub struct Prop {
        directive: kw::prop,
        name: syn::Ident,
        value: Value,
    }
}

create_directive! {
    use parse_ident_braced for pub struct Clone {
        directive: kw::clone,
        name: syn::Ident,
        value: Value,
    }
}

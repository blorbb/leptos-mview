mod children;
pub use children::*;
mod element;
pub use element::*;
pub mod attribute;
pub use attribute::{Attr, Attrs};
mod ident;
pub use ident::*;
mod tag;
pub use tag::*;
mod value;
pub use value::*;

macro_rules! derive_multi_ast_for {
    {
        struct $new:ident(Vec<$inner:ty>);
    } => {
        impl ::std::ops::Deref for $new {
            type Target = [$inner];

            fn deref(&self) -> &Self::Target { &self.0 }
        }
    };

    {
        struct $new:ident(Vec<$inner:ty>);
        impl Parse(non_empty_error = $err:literal);
    } => {
        derive_multi_ast_for! { struct $new(Vec<$inner>); }

        impl ::syn::parse::Parse for $new {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                let mut vec = Vec::new();
                while let Ok(inner) = input.parse::<$inner>() {
                    vec.push(inner);
                }

                if !input.is_empty() {
                    ::proc_macro_error::abort!(input.span(), $err);
                };
                Ok(Self(vec))
            }
        }
    };

    {
        struct $new:ident(Vec<$inner:ty>);
        impl Parse(allow_non_empty);
    } => {
        derive_multi_ast_for! { struct $new(Vec<$inner>); }

        impl ::syn::parse::Parse for $new {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                let mut vec = Vec::new();
                while let Ok(inner) = input.parse::<$inner>() {
                    vec.push(inner);
                }

                Ok(Self(vec))
            }
        }
    }
}

pub(crate) use derive_multi_ast_for;

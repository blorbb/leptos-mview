//! Custom ASTs.
//!
//! Only 'basic' ASTs (values, idents, etc.) with one obvious way of expanding
//! them have a [`ToTokens`](quote::ToTokens) implementation. Other ASTs with
//! context-specific expansions (like expanding differently in components or
//! HTML elements) or complex ASTs have their expansion implementations in
//! [`crate::expand`].
//!
//! Most ASTs also implement [`Parse`](syn::parse::Parse). 'Basic' ASTs will
//! return an [`Err`] if they fail to parse, and do not advance the
//! [`ParseStream`](syn::parse::ParseStream). More complicated ASTs with clear
//! syntax requirements will [`abort`](proc_macro_error::abort) if parsing
//! fails. See each AST for details about whether parsing errors or aborts.

pub mod attribute;
pub use attribute::{Attr, Attrs};
mod children;
pub use children::*;
mod element;
pub use element::*;
mod ident;
pub use ident::*;
mod tag;
pub use tag::*;
mod value;
pub use value::*;

/// Macro for deriving an AST that contains multiple inner ASTs.
///
/// This does not create the AST itself to allow other `#[derive]`s or doc
/// comments.
///
/// # Usage
/// There are three forms:
/// 1. Plain struct.
/// ```ignore
/// pub struct Children(Vec<Child>);
/// derive_multi_ast_for! { struct Children(Vec<Child>); }
/// ```
///
/// This only implements [`Deref<Target = [Child]>`](std::ops::Deref). The two
/// other forms below also get this.
///
/// 2. With parsing.
/// ```ignore
/// pub struct Children(Vec<Child>);
/// derive_multi_ast_for! {
///     struct Children(Vec<Child>);
///     impl Parse(allow_non_empty);
/// }
/// ```
///
/// This adds a [`syn::parse::Parse`] implementation that tries to parse as many
/// `Child`s as possible (without separators), and adds it to the stored
/// [`Vec`]. This includes parsing zero `Child`s, which will make an empty
/// `Vec`.
///
/// 3. With parsing, requiring all tokens are parsed.
/// ```ignore
/// pub struct Children(Vec<Child>);
/// derive_multi_ast_for! {
///     struct Children(Vec<Child>);
///     impl Parse(non_empty_error = "error message");
/// }
/// ```
///
/// Same as `2.`, but adds a check to make sure that there are no more tokens in
/// the [`ParseStream`](syn::parse::ParseStream). If there are, parsing will
/// **abort** with the provided error message.
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
                    ::proc_macro_error::emit_error!(input.span(), $err);
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

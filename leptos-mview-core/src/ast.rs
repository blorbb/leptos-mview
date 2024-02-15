//! Custom ASTs.
//!
//! Only 'basic' ASTs (values, idents, etc.) with one obvious way of expanding
//! them have a [`ToTokens`](quote::ToTokens) implementation. Other ASTs with
//! context-specific expansions (like expanding differently in components or
//! HTML elements) or complex ASTs have their expansion implementations in
//! [`crate::expand`].
//!
//! Most ASTs also implement [`Parse`](syn::parse::Parse). The point at which it
//! errors should not be relied on - i.e. do not run:
//! ```ignore
//! if let Ok(x) = X::parse(input) { /* ... */ }
//! ````
//! Instead, use [`rollback_err`](crate::recover::rollback_err) to avoid
//! advancing the input if parsing fails. Note that some parse implementations
//! use [`proc_macro_error::emit_error`] to try and recover from incomplete
//! expressions, so use [`input.peek`](syn::parse::ParseBuffer::peek) where
//! possible if layered errors are not desired.

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

//! `proc_macro_error` has not updated for `syn` v2, so the
//! `.unwrap_or_abort()` and related extension methods do not work.
//!
//! A simplified version of the extension traits have been added here.

use proc_macro_error2::{abort, emit_error};

pub trait ResultExt {
    type Ok;

    /// Behaves like `Result::unwrap`: if self is `Ok` yield the contained
    /// value, otherwise abort macro execution via `abort!`.
    fn unwrap_or_abort(self) -> Self::Ok;
}

impl<T> ResultExt for Result<T, syn::Error> {
    type Ok = T;

    fn unwrap_or_abort(self) -> T {
        match self {
            Ok(res) => res,
            Err(e) => abort!(e.span(), e.to_string()),
        }
    }
}

pub trait SynErrorExt {
    fn emit_as_error(self);
}

impl SynErrorExt for syn::Error {
    fn emit_as_error(self) { emit_error!(self.span(), "{}", self) }
}

//! `proc_macro_error` has not updated for `syn` v2, so the
//! `.unwrap_or_abort()` and related extension methods do not work.
//!
//! A simplified version of the extension traits have been added here.

use proc_macro_error::{abort, abort_call_site, emit_error};

pub trait ResultExt {
    type Ok;

    /// Behaves like `Result::unwrap`: if self is `Ok` yield the contained
    /// value, otherwise abort macro execution via `abort!`.
    fn unwrap_or_abort(self) -> Self::Ok;

    /// Behaves like `Result::expect`: if self is `Ok` yield the contained
    /// value, otherwise abort macro execution via `abort!`.
    /// If it aborts then resulting error message will be preceded with
    /// `message`.
    fn expect_or_abort(self, msg: &str) -> Self::Ok;

    /// Behaves like `expect_or_abort` but the existing error message is
    /// overwritten, not appended to the new one.
    fn expect_or_abort_with_msg(self, message: &str) -> Self::Ok;
}

/// This traits expands `Option` with some handy shortcuts.
pub trait OptionExt {
    type Some;

    /// Behaves like `Option::expect`: if self is `Some` yield the contained
    /// value, otherwise abort macro execution via `abort_call_site!`.
    /// If it aborts the `message` will be used for
    /// [`compile_error!`][compl_err] invocation.
    ///
    /// [compl_err]: https://doc.rust-lang.org/std/macro.compile_error.html
    fn expect_or_abort(self, msg: &str) -> Self::Some;
}

impl<T> ResultExt for Result<T, syn::Error> {
    type Ok = T;

    fn unwrap_or_abort(self) -> T {
        match self {
            Ok(res) => res,
            Err(e) => abort!(e.span(), e.to_string()),
        }
    }

    fn expect_or_abort(self, message: &str) -> T {
        match self {
            Ok(res) => res,
            Err(e) => {
                let msg = format!("{message}: {e}");
                abort!(e.span(), msg)
            }
        }
    }

    fn expect_or_abort_with_msg(self, message: &str) -> T {
        match self {
            Ok(res) => res,
            Err(e) => abort!(e.span(), message),
        }
    }
}

impl<T> OptionExt for Option<T> {
    type Some = T;

    fn expect_or_abort(self, message: &str) -> T {
        match self {
            Some(res) => res,
            None => abort_call_site!(message),
        }
    }
}

pub trait SynErrorExt {
    fn emit_as_error(self);
}

impl SynErrorExt for syn::Error {
    fn emit_as_error(self) { emit_error!(self.span(), "{}", self) }
}

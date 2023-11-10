//! Mini helper functions for working with spans.

use proc_macro2::Span;

/// Tries to join two spans together, returning just the first span if
/// unable to join.
///
/// The spans are unable to join if the user is not on nightly or the spans
/// are in different files.
pub fn join(s1: Span, s2: Span) -> Span { s1.join(s2).unwrap_or(s1) }

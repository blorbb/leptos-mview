use proc_macro2::Span;

/// Tries to join two spans together, returning just the first span if
/// unable to join.
pub fn join(s1: Span, s2: Span) -> Span { s1.join(s2).unwrap_or(s1) }

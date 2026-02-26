use leptos::prelude::*;
use leptos_mview::mview;

// NOTE: there is an 'info' underline under the `mview` macro because one is
// given at the start of `::leptos::tachys::html::element::span()`.
// it looks bad in the UI test but is basically nothing in rust-analyzer.
// imo, better to have `span` give more relevant hover hints (just the function)
// than to fix this little spanning issue.
fn invalid_value() {
    _ = mview! {
        div class:x={true} {
            span class=test
        }
    }
}

fn incomplete_directive() {
    _ = mview! {
        div class:x={true} {
            span class:
        }
    }
}

fn main() {}

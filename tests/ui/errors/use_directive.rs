use leptos::{prelude::*, web_sys::Element};
use leptos_mview::mview;

fn no_arg_dir(_el: Element) {}

fn arg_dir(_el: Element, _argument: i32) {}

fn missing_argument() {
    _ = mview! {
        div use:arg_dir;
    };
}

// the span looks bad on this but it's just a light info about the into call.
// it looks fine in rust-analyzer.
fn extra_argument() {
    _ = mview! {
        span use:no_arg_dir=2;
    };
}

fn main() {}

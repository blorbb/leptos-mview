use leptos::{html::AnyElement, HtmlElement};
use leptos_mview::mview;

fn no_arg_dir(_el: HtmlElement<AnyElement>) {}

fn arg_dir(_el: HtmlElement<AnyElement>, _argument: i32) {}

fn missing_argument() {
    _ = mview! {
        div use:arg_dir;
    };
}

fn extra_argument() {
    _ = mview! {
        span use:no_arg_dir=2;
    };
}

fn main() {}

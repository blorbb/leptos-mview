use leptos::{html::AnyElement, HtmlElement};
use leptos_mview::view;

fn no_arg_dir(_el: HtmlElement<AnyElement>) {}

fn arg_dir(_el: HtmlElement<AnyElement>, _argument: i32) {}

fn missing_argument() {
    _ = view! {
        div use:arg_dir;
    };
}

fn extra_argument() {
    _ = view! {
        span use:no_arg_dir=2;
    };
}

fn main() {}

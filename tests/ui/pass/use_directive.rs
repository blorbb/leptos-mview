use leptos::{*, html::AnyElement};
use leptos_mview::view;

fn no_arg_dir(_el: HtmlElement<AnyElement>) {}

fn arg_dir(_el: HtmlElement<AnyElement>, _argument: i32) {}

fn main() {
    _ = view! {
        div use:no_arg_dir {
            span use:arg_dir=10;
        }
    };

    _ = view! {
        Component use:no_arg_dir;
        Component use:arg_dir=300;
    };
}

#[component]
fn Component() -> impl IntoView {
    view! { button { "hi" } }
}

#[component]
fn Spreadable(#[prop(attrs)] attrs: Vec<(&'static str, Attribute)>) -> impl IntoView {
    view! {
        div {..attrs};
    }
}

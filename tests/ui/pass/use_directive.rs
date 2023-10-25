use leptos::{*, html::AnyElement};
use leptos_mview::mview;

fn no_arg_dir(_el: HtmlElement<AnyElement>) {}

fn arg_dir(_el: HtmlElement<AnyElement>, _argument: i32) {}

fn main() {
    _ = mview! {
        div use:no_arg_dir {
            span use:arg_dir=10;
        }
    };

    _ = mview! {
        Component use:no_arg_dir;
        Component use:arg_dir=300;
    };
}

#[component]
fn Component() -> impl IntoView {
    mview! { button { "hi" } }
}

#[component]
fn Spreadable(#[prop(attrs)] attrs: Vec<(&'static str, Attribute)>) -> impl IntoView {
    mview! {
        div {..attrs};
    }
}

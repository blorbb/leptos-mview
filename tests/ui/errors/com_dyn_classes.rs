use leptos::{prelude::*, text_prop::TextProp};
use leptos_mview::mview;

#[component]
fn AComponent(
    #[prop(into, default="".into())] class: TextProp,
    #[prop(optional)] id: &'static str,
) -> impl IntoView {
    mview! {
        div class=f["my-class {}", class.get()] {id};
    }
}

fn incorrect_type() {
    _ = mview! {
        AComponent class:red=["not this"];
    };
}

fn main() {}

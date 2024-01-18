use leptos::*;
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

fn missing_closure() {
    _ = mview! {
        AComponent class:red=true;
    };
}

fn incorrect_type() {
    _ = mview! {
        AComponent class:red=["not this"];
    };
}

#[component]
fn Nothing() -> impl IntoView {}

// these spans are actually fine, there's a blank info message at `mview!` for
// some reason.

fn no_attribute_reactive() {
    _ = mview! {
        Nothing class:red=[true];
    };
}

fn no_attribute_static() {
    _ = mview! {
        Nothing.red;
    };
}

fn no_attribute_id() {
    _ = mview! {
        Nothing #unique;
    };
}

fn main() {}

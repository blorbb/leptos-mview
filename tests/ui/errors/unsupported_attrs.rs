use leptos::*;
use leptos_mview::mview;

fn style_on_component() {
    mview! {
        Component style:color="white";
    };
}

fn prop_on_component() {
    mview! {
        Component prop:value="1";
    };
}

fn attr_on_element() {
    mview! {
        input attr:class="no" type="text";
    };
}

fn clone_on_element() {
    let notcopy = String::new();
    mview! {
        div {
            span clone:notcopy {
                {notcopy.clone()}
            }
        }
    };
}

#[component]
fn Component() -> impl IntoView {
    mview! {
        button;
    };
}

fn main() {}

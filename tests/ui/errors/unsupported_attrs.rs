use leptos::prelude::*;
use leptos_mview::mview;

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

fn main() {}

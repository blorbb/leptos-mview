//! Testing that there are no errors that cause the entire macro to error (i.e.
//! call-site error)

use leptos::*;
use leptos_mview::mview;

fn missing_args() {
    // missing `key` attribute
    _ = mview! {
        For each=[[1, 2, 3]] |i| { {i} }
    };
}

fn incorrect_arg_value() {
    // Show takes `bool` not `&str`
    _ = mview! {
        Show when={"no"} {
            "hi"
        }
    };
}

fn missing_closure() {
    _ = mview! {
        Show when={true} {
            "hi"
        }
    };
}

fn incorrect_closure() {
    #[component]
    fn Thing(label: &'static str) -> impl IntoView { label }

    // `label` is not a closure
    _ = mview! {
        Thing label=[false];
    };
}

fn incorrect_closure_to_children() {
    #[component]
    fn Thing(children: Children) -> impl IntoView { children() }

    let s = String::new();
    // `children` does not take a closure
    _ = mview! {
        Thing |s| { "hello" }
    };
}

fn missing_closure_to_children() {
    // thought it would make an error at the `.children`, but it seem to accept it
    // and errors at the tag name instead. this test is just for notifying in case
    // this ever changes.
    _ = mview! {
        Await future=[async { 3 }] { "no args" }
    };
}

fn main() {}

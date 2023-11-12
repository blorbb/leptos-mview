//! Testing that there are no errors that cause the entire macro to error (i.e.
//! call-site error).
//! 
//! This file is for testing on the slot itself, see `com_builder_spans` for testing on components.

use leptos::*;
use leptos_mview::mview;

#[slot]
struct SChildren {
    an_attr: i32,
    children: ChildrenFn,
}

#[component]
fn TakesSChildren(s_children: SChildren) -> impl IntoView { let _ = s_children; }

fn missing_args() {
    _ = mview! {
        TakesSChildren {
            slot:SChildren { "hi" }
        }
    };
}

fn incorrect_arg_value() {
    _ = mview! {
        TakesSChildren { slot:SChildren an_attr="no" { "what" } }
    };
}

fn incorrect_closure_to_children() {
    let s = String::new();
    _ = mview! {
        TakesSChildren {
            slot:SChildren an_attr=1 |s| { "this is " {s} }
        }
    };
}

#[slot]
struct SNoChildren {
    an_attr: i32,
}

#[component]
fn TakesSNoChildren(s_no_children: SNoChildren) -> impl IntoView { let _ = s_no_children; }

fn incorrect_children() {
    _ = mview! {
        TakesSNoChildren {
            slot:SNoChildren an_attr=5 { "hey!" }
        }
    };
}

#[slot]
struct SClosureChildren {
    children: Callback<i32, View>,
}

#[component]
fn TakesSClosureChildren(s_closure_children: SClosureChildren) -> impl IntoView { let _ = s_closure_children; }

fn missing_closure_to_children() {
    _ = mview! {
        TakesSClosureChildren {
            slot:SClosureChildren { "hey!" }
        }
    };
}


fn main() {}

//! Test allowing parentheses to wrap the children as well.

use leptos::prelude::*;
use leptos_mview::mview;
mod utils;
use utils::check_str;

#[test]
fn html_child() {
    let res = mview! {
        strong("child")
        "go" em("to" a href="#" ("nowhere"))
    };

    check_str(
        res,
        ["child</strong>go<em", "to<a href=\"#\"", "nowhere</a></em>"].as_slice(),
    )
}

#[test]
fn component_closure() {
    if false {
        _ = mview! {
            For each={|| [1, 2, 3]}
                key={|i| *i}
            |index| (
                "i is " {index}
            )
        };
    }
}

#[test]
fn component_child() {
    if false {
        _ = mview! {
            Show
                when={|| true}
            (
                "hello!"
            )
        };
    }
}

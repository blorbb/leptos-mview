//! Testing ground for macro expansions

use leptos::*;

fn main() {
    view! {
        <div>
            "a"
            <span></span>
        </div>
        <div></div>
    };

    "2";
    view! {
        <div></div>
    };

    "3";
    view! {
        "hi"
        <div></div>
    };
    "4";
    view! { "aaa" };
}
use leptos::*;
use leptos_mview::mview;
mod utils;
use utils::check_str;

// #[test]
// fn strings() {
//     let result: &str = mview! {
//         "hello there!"
//     };
//     assert_eq!(result, "hello there!");
// }

// cannot traverse the DOM as there is no browser
// so I am testing in a way similar to
// https://github.com/leptos-rs/leptos/blob/main/leptos/tests/ssr.rs

#[test]
fn single_element() {
    let result: HtmlElement<html::Div> = mview! {
        div {
            "hi"
        }
    };
    check_str(result, r#"<div data-hk="0-0-0-1">hi</div>"#);
}

#[test]
fn multi_element_is_fragment() {
    let _fragment: Fragment = mview! {
        div { "a" }
        span { "b" }
    };
}

#[test]
fn a_bunch() {
    let result = mview! {
        "hi"
        span class="abc" data-index={0} {
            strong { "d" }
            {3}
        }
        br;
        input type="checkbox" checked;
    };

    check_str(
        result,
        "hi\
        <span class=\"abc\" data-index=\"0\" data-hk=\"0-0-0-2\">\
            <strong data-hk=\"0-0-0-3\">d</strong>\
            3\
        </span>\
        <br data-hk=\"0-0-0-4\"/>\
        <input type=\"checkbox\" checked data-hk=\"0-0-0-5\"/>",
    );
}

#[test]
fn directive_before_attr() {
    let result = mview! {
        span class:exist=true class="dont override";
    };
    check_str(result, "dont override exist");

    let result = mview! {
        span style:color="black" style="font-size: 1em;";
    };
    check_str(result, "font-size: 1em; color: black;");
}

#[test]
fn multiple_directives() {
    let yes = move || true;
    let no = move || false;
    let color = move || "white";
    let result = mview! {
        div
            class:here={yes}
            style:color={color}
            class:not={no}
            class:also-here=true
            class="normal"
            style="line-height: 1.5;"
            style:background-color="red";
    };

    check_str(
        result,
        r#"class="normal here also-here" style="line-height: 1.5; color: white; background-color: red;""#,
    );
}

#[test]
fn string_directives() {
    let yes = move || true;
    let result = mview! {
        div
            class:"complex[class]-name"={yes}
            style:"doesn't-exist"="black"
            class:"not-here"=false;
    };

    check_str(
        result,
        r#"class="complex[class]-name" style="doesn't-exist: black;""#,
    )
}

#[test]
fn mixed_class_creation() {
    let class: TextProp = "some-class another-class".into();
    let r = mview! {
        div.always-here class=[class.get()];
    };

    check_str(r, r#"class="some-class another-class always-here""#);
}

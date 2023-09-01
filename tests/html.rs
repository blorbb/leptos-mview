use leptos::*;
use leptos_mview::view;

#[test]
fn strings() {
    let result: &str = view! {
        "hello there!"
    };
    assert_eq!(result, "hello there!");
}

// cannot traverse the DOM as there is no browser
// so I am testing in a way similar to
// https://github.com/leptos-rs/leptos/blob/main/leptos/tests/ssr.rs

#[test]
fn single_element() {
    let result: HtmlElement<html::Div> = view! {
        div {
            "hi"
        }
    };
    assert_eq!(
        result.into_view().render_to_string(),
        r#"<div id="_0-0-1">hi</div>"#
    );
}

#[test]
fn a_bunch() {
    let result = view! {
        "hi"
        span class="abc" data-index={0} {
            strong { "d" }
            {3}
        }
        br;
        input type="checkbox" checked;
    };
    assert!(result.into_view().render_to_string().contains(
        "hi\
        <span class=\"abc\" data-index=\"0\" id=\"_0-0-2\">\
            <strong id=\"_0-0-3\">d</strong>3\
        </span>\
        <br id=\"_0-0-4\"/>\
        <input type=\"checkbox\" checked id=\"_0-0-5\"/>"
    ))
}

use leptos::*;
use leptos_mview::view;

#[track_caller]
fn check_str(component: impl IntoView, contains: &str) {
    let component_str = component.into_view().render_to_string();
    assert!(
        component_str.contains(contains),
        "expected \"{contains}\" to be found in the component render.\n\
        Found:\n\
        {component_str}"
    )
}

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
    check_str(result, r#"<div id="_0-0-1">hi</div>"#);
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

    check_str(
        result,
        "hi\
        <span class=\"abc\" data-index=\"0\" id=\"_0-0-2\">\
            <strong id=\"_0-0-3\">d</strong>3\
        </span>\
        <br id=\"_0-0-4\"/>\
        <input type=\"checkbox\" checked id=\"_0-0-5\"/>",
    );
}

#[test]
fn directive_before_attr() {
    let result = view! {
        span class:exist=true class="dont override";
    };
    check_str(result, "dont override exist");

    let result = view! {
        span style:color="black" style="font-size: 1em;";
    };
    check_str(result, "font-size: 1em; color: black;");
}

#[test]
fn multiple_directives() {
    let yes = move || true;
    let no = move || false;
    let color = move || "white";
    let result = view! {
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

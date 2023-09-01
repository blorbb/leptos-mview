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
    // assert_eq!(
    //     result.into_view().render_to_string(),
    //     r#"<div id="_0-0-1">hi</div>"#
    // );
    let binding = result.into_view().render_to_string();
    let dom = tl::parse(&binding, Default::default()).unwrap();
    let parser = dom.parser();
    assert_eq!(dom.children().len(), 1);
    assert_eq!(
        dom.children()
            .first()
            .unwrap()
            .get(parser)
            .unwrap()
            .inner_text(parser),
        "hi"
    )
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
    let binding = result.into_view().render_to_string();
    let dom = tl::parse(&binding, Default::default()).unwrap();
    let parser = dom.parser();

    let mut it = dom.nodes().into_iter();
    (|| -> Option<_> {
        let text = it.find(|node| node.as_raw().is_some())?.outer_html(parser);
        assert_eq!(text, "hi");

        let span = it.next()?.as_tag()?;
        assert_eq!(span.name(), "span");
        assert_eq!(span.attributes().class()?, "abc");
        assert_eq!(span.attributes().get("data-index")??, "0");

        let strong = it.next()?.as_tag()?;
        assert_eq!(strong.name(), "strong");

        let text = it.next()?.as_raw()?;
        assert_eq!(text, "d");

        let text = it.next()?.as_raw()?;
        assert_eq!(text, "3");

        let br = it.next()?.as_tag()?;
        assert_eq!(br.name(), "br");

        let input = it.next()?.as_tag()?;
        assert_eq!(input.name(), "input");
        assert_eq!(input.attributes().get("type")??, "checkbox");
        assert!(input.attributes().get("checked")?.is_none());
        Some(())
    })()
    .unwrap();
}

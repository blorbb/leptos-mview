use leptos::prelude::*;
use leptos_mview::mview;
mod utils;
use utils::check_str;

#[test]
fn spread_html_element() {
    let attrs = view! { <{..} data-index=0 class="c" data-another="b" /> };
    let res = mview! {
        div {..attrs} data-yet-another-thing="z" {
            "children"
        }
    };
    check_str(
        res,
        r#"<div data-yet-another-thing="z" data-index="0" data-another="b" class="c">children</div>"#,
    );
}

#[test]
fn spread_in_component() {
    #[component]
    fn Spreadable() -> impl IntoView {
        mview! {
            div;
        }
    }

    let res = mview! {
        Spreadable attr:class="b" attr:contenteditable=true attr:data-index=0;
    };
    check_str(
        res,
        r#"<div contenteditable data-index="0" class="b"></div>"#,
    );
}

#[test]
fn spread_on_component() {
    #[component]
    fn Spreadable() -> impl IntoView {
        mview! {
            div;
        }
    }

    let attrs = view! { <{..} data-a="b" data-index=0 class="c" /> };

    let res = mview! {
        Spreadable attr:contenteditable=true {..attrs};
    };
    check_str(
        res,
        r#"<div contenteditable data-a="b" data-index="0" class="c"></div>"#,
    );
}

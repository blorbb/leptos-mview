use leptos::*;
use leptos_mview::mview;
mod utils;
use utils::check_str;

#[test]
fn spread_html_element() {
    let attrs: Vec<(&'static str, Attribute)> = vec![
        ("a", "b".into_attribute()),
        ("data-index", 0.into_attribute()),
        ("class", "c".into_attribute()),
    ];
    let res = mview! {
        div {..attrs} class="b" {
            "children"
        }
    };
    check_str(
        res,
        r#"<div class="b" a="b" data-index="0" class="c" data-hk="0-0-0-1">children</div>"#,
    );
}

#[test]
fn spread_on_component() {
    #[component]
    fn Spreadable(#[prop(attrs)] attrs: Vec<(&'static str, Attribute)>) -> impl IntoView {
        mview! {
            div {..attrs};
        }
    }

    let res = mview! {
        Spreadable attr:class="b" attr:contenteditable=true attr:data-index=0;
    };
    check_str(
        res,
        r#"<div class="b" contenteditable data-index="0" data-hk="0-0-0-2"></div>"#,
    );
}

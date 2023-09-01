use leptos::*;
use leptos_mview::view;

fn main() {
    // type will be a Fragment if it fails
    let _div: HtmlElement<html::Div> = view! {
        div { "a" }
    };
}

use leptos::prelude::*;
use leptos_mview::mview;
use utils::check_str;
mod utils;

#[test]
fn f_value() {
    let yes = || true;
    let no = || false;
    let r = mview! {
        div aria-selected=f["{}", yes()] data-not-selected=f["{}", no()];
    };

    check_str(r, r#"<div aria-selected="true" data-not-selected="false""#);

    let number = 2.12545;
    let r = mview! {
        input type="number" value=f["{number:.2}"];
    };
    check_str(r, r#"<input type="number" value="2.13""#);
}

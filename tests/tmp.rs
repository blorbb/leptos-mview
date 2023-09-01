//! Testing ground for macro expansions

use leptos::*;
use leptos_mview_macro::view as mview;

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
    "5";
    let s = create_rw_signal("a".to_string());
    // _ = view! {
    //     <div>
    //         <Comp key=3 signal={s}>
    //             "a"
    //             <div>"b"</div>
    //         </Comp>
    //     </div>
    // };
    // #[component]
    // fn Comp(key: i32, #[into] signal: Signal<String>, children: Children) -> impl IntoView {
    //     view! {
    //         <div>"inside component" {key} {children()}</div>
    //     }
    // }
    _ = view! {
        "a"
        {s}
        {move || s}
        {3}
        "b"
    };
    mview! {
        "a" {s} (move || s) {3} "b"
    };
}

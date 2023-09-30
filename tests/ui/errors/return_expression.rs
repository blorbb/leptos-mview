use leptos_mview::view;

fn main() {
    // should not get an "unexpected end of macro invocation"
    let expr = view! {
        div class=;
    };
}
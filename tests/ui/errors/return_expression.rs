use leptos_mview::mview;

fn main() {
    // should not get an "unexpected end of macro invocation"
    let expr = mview! {
        div class=;
    };
}
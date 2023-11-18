use leptos::*;
use leptos_mview::mview;

fn main() {
    mview! {
        Await
            future=[async { 1 }]
        |data| "no"
    };
}
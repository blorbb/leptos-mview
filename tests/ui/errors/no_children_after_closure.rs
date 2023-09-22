use leptos_mview::view;

fn main() {
    view! {
        Await
            future=[async { 1 }]
        |data| "no"
    };
}
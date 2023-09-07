use leptos::*;
use leptos_mview::view;

// TODO: not sure why this is creating an untracked resource warning
fn main() {
    _ = view! {
        Await future={|| async { 3 }} |data| {
            p { {*data} " little monkeys, jumping on the bed." }
        }
    };

    // clone should also work
    let name = String::new();
    _ = view! {
        Await
            future={move || async {"hi".to_string()}}
            clone:name={name}
        |greeting| {
            {greeting} " " {name.clone()}
        }
    };
}

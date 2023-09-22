use leptos::*;
use leptos_mview::view;

#[test]
fn clones() {
    #[component]
    fn Owning(children: ChildrenFn) -> impl IntoView {
        view! { div { {children} } }
    }

    let notcopy = String::new();
    _ = view! {
        Owning {
            Owning clone:{notcopy} {
                {notcopy.clone()}
            }
        }
    };
}

// TODO: not sure why this is creating an untracked resource warning
#[test]
fn children_args() {
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
            clone:{name}
        |greeting| {
            {greeting} " " {name.clone()}
        }
    };
}

fn main() {}

use leptos::*;
use leptos_mview::view;

fn main() {
    let non_copy = String::new();
    _ = view! {
        Owning {
            Owning clone:non_copy={non_copy} {
                {non_copy.clone()}
            }
        }
    };
}

#[component]
fn Owning(children: ChildrenFn) -> impl IntoView {
    // TODO: replace with `{children}` when fixed in rc2
    view! {
        div { {children()} }
    }
}

use leptos::*;
use leptos_mview::view;
mod utils;
use utils::check_str;

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

#[test]
fn generics() {
    use core::marker::PhantomData;
    // copied from https://github.com/leptos-rs/leptos/pull/1636
    #[component]
    pub fn GenericComponent<S>(ty: PhantomData<S>) -> impl IntoView {
        let _ty = ty;
        std::any::type_name::<S>()
    }

    let result = view! {
        GenericComponent<String> ty={PhantomData};
        GenericComponent<usize> ty={PhantomData};
        GenericComponent<i32> ty={PhantomData};
    };

    check_str(result, ["alloc::string::String", "usize", "i32"].as_slice());
}

fn main() {}

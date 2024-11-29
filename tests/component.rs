use leptos::{prelude::*, task::Executor};
use leptos_mview::mview;
mod utils;
use utils::check_str;

#[test]
fn basic() {
    #[component]
    fn MyComponent(
        my_attribute: &'static str,
        another_attribute: Vec<i32>,
        children: Children,
    ) -> impl IntoView {
        mview! {
            div class="my-component" data-my-attribute={my_attribute} data-another=f["{another_attribute:?}"] {
                {children()}
            }
        }
    }

    _ = view! {
        <MyComponent my_attribute="something" another_attribute=vec![0, 1]>
            "my child"
        </MyComponent>
    }
}

#[test]
fn clones() {
    #[component]
    fn Owning(children: ChildrenFn) -> impl IntoView {
        mview! { div { {children()} } }
    }

    let notcopy = String::new();
    _ = mview! {
        Owning {
            Owning clone:notcopy {
                {notcopy.clone()}
            }
        }
    };
}

// TODO: not sure why this is creating an untracked resource warning
#[test]
fn children_args() {
    Executor::init_futures_executor().unwrap();
    _ = mview! {
        Await future={async { 3 }} |data| {
            p { {*data} " little monkeys, jumping on the bed." }
        }
    };

    // clone should also work
    let name = String::new();
    _ = mview! {
        Await
            future={async {"hi".to_string()}}
            clone:name
        |greeting| {
            {greeting.clone()} " " {name.clone()}
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

    let result = mview! {
        GenericComponent<String> ty={PhantomData};
        GenericComponent<usize> ty={PhantomData};
        GenericComponent<i32> ty={PhantomData};
    };

    check_str(result, ["alloc::string::String", "usize", "i32"].as_slice());

    // also accept turbofish
    let result = mview! {
        GenericComponent::<String> ty={PhantomData};
        GenericComponent::<usize> ty={PhantomData};
        GenericComponent::<i32> ty={PhantomData};
    };

    check_str(result, ["alloc::string::String", "usize", "i32"].as_slice());
}

#[test]
fn qualified_paths() {
    let _result = mview! {
        leptos::control_flow::Show when=[true] {
            "a"
        }
        leptos::control_flow::Show when=[false] {
            "b"
        }
    };

    // requires ssr feature to check the output
    // check_str(result, Contains::AllOfNoneOf([&["a"], &["b"]]))
}

// don't try parse slot:: as a slot
mod slot {
    use leptos::*;

    #[component]
    pub fn NotASlot() -> impl IntoView {}
}

#[test]
fn slot_peek() {
    _ = mview! {
        slot::NotASlot;
    }
}

#[test]
fn let_patterns() {
    if false {
        let letters = ['a', 'b', 'c'];
        _ = mview! {
            For
                each=[letters.into_iter().enumerate()]
                key={|(i, _)| *i}
            |(i, letter)| {
                "letter " {i+1} " is " {letter}
            }
        };
    }
}

#[component]
fn TakesClass() -> impl IntoView {
    mview! {
        div class="takes-class" {
            "I take more classes!"
        }
    }
}

#[component]
fn TakesIds() -> impl IntoView {
    mview! {
        div class="i-take-ids";
    }
}

#[test]
fn selectors() {
    let r = mview! {
        TakesClass.test1.test-2;
    };

    check_str(r, r#"<div class="takes-class test1 test-2"#)
}

// untracked signal warning... should be fine.
#[test]
fn class_dir() {
    let yes = RwSignal::new(true);
    let no = move || !yes.get();
    let r = mview! {
        TakesClass.test1.test-2 class:not-this={no} class:this=[yes.get()] class:"complicated"=[yes.get()];
    };
    check_str(
        r,
        r#"div class="takes-class test1 test-2  this complicated""#,
    );
}

#[test]
fn ids() {
    let r = mview! {
        TakesIds #id-1 #id-number-two;
    };

    check_str(r, r#"<div id="id-1 id-number-two" class="i-take-ids""#)
}

use leptos::*;
use leptos_mview::mview;
mod utils;
use utils::{check_str, Contains};

#[test]
fn clones() {
    #[component]
    fn Owning(children: ChildrenFn) -> impl IntoView {
        mview! { div { {children} } }
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
    _ = mview! {
        Await future={|| async { 3 }} |data| {
            p { {*data} " little monkeys, jumping on the bed." }
        }
    };

    // clone should also work
    let name = String::new();
    _ = mview! {
        Await
            future={move || async {"hi".to_string()}}
            clone:name
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
    let result = mview! {
        leptos::Show when=[true] {
            "a"
        }
        leptos::Show when=[false] {
            "b"
        }
    };

    check_str(result, Contains::AllOfNoneOf([&["a"], &["b"]]))
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
fn TakesClass(#[prop(into)] class: TextProp) -> impl IntoView {
    mview! {
        div class=f["takes-class {}", class.get()] {
            "I take more classes!"
        }
    }
}

#[component]
fn TakesIds(id: &'static str) -> impl IntoView {
    mview! {
        div {id} class="i-take-ids";
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
    let runtime = create_runtime();
    let yes = RwSignal::new(true);
    let no = move || !yes();
    let r = mview! {
        TakesClass.test1.test-2 class:not-this={no} class:this={yes} class:"complicated"=[yes()];
    };
    check_str(
        r,
        r#"div class="takes-class test1 test-2 this complicated""#,
    );

    runtime.dispose();
}

#[test]
fn ids() {
    let r = mview! {
        TakesIds #id-1 #id-number-two;
    };

    check_str(r, r#"<div id="id-1 id-number-two" class="i-take-ids""#)
}

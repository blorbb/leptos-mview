use leptos::*;
use leptos_mview::mview;
use utils::check_str;
mod utils;

// same example as the one given in the #[slot] proc macro documentation.

#[slot]
struct HelloSlot {
    #[prop(optional)]
    children: Option<Children>,
}

#[component]
fn HelloComponent(hello_slot: HelloSlot) -> impl IntoView {
    if let Some(children) = hello_slot.children {
        (children)().into_view()
    } else {
        ().into_view()
    }
}

#[test]
fn test_example() {
    let r = mview! {
        HelloComponent {
            slot:HelloSlot {
                "Hello, World!"
            }
        }
    };

    utils::check_str(r, "Hello, World!");
}

// https://github.com/leptos-rs/leptos/blob/main/examples/slots/src/lib.rs

#[slot]
struct Then {
    children: ChildrenFn,
}

#[slot]
struct ElseIf {
    #[prop(into)]
    cond: MaybeSignal<bool>,
    children: ChildrenFn,
}

#[slot]
struct Fallback {
    children: ChildrenFn,
}

// Slots are added to components like any other prop.
#[component]
fn SlotIf(
    #[prop(into)] cond: MaybeSignal<bool>,
    then: Then,
    #[prop(default=vec![])] else_if: Vec<ElseIf>,
    #[prop(optional)] fallback: Option<Fallback>,
) -> impl IntoView {
    move || {
        if cond() {
            (then.children)().into_view()
        } else if let Some(else_if) = else_if.iter().find(|i| (i.cond)()) {
            (else_if.children)().into_view()
        } else if let Some(fallback) = &fallback {
            (fallback.children)().into_view()
        } else {
            ().into_view()
        }
    }
}

#[test]
pub fn multiple_slots() {
    for (count, ans) in [(0, "even"), (5, "x5"), (45, "x5"), (9, "odd"), (7, "x7")] {
        let is_even = count % 2 == 0;
        let is_div5 = count % 5 == 0;
        let is_div7 = count % 7 == 0;

        let r = mview! {
            SlotIf cond={is_even} {
                slot:Then { "even" }
                slot:ElseIf cond={is_div5} { "x5" }
                slot:ElseIf cond={is_div7} { "x7" }
                slot:Fallback { "odd" }
            }
        };

        check_str(r, ans);
    }
}

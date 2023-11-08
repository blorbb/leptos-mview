use leptos::*;
use leptos_mview::mview;
use utils::{check_str, Contains};
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

    check_str(r, "Hello, World!");
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

#[component]
fn SlotIf(
    #[prop(into)] cond: MaybeSignal<bool>,
    then: Then,
    #[prop(optional)] else_if: Vec<ElseIf>,
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

#[test]
pub fn accept_multiple_use_single() {
    // else_if takes Vec<ElseIf>, check if just giving a single one
    // (which should just pass a single ElseIf instead of a vec)
    // still works
    let r = mview! {
        SlotIf cond=false {
            slot:Then { "no!" }
            slot:ElseIf cond=true { "yes!" }
            slot:Fallback { "absolutely not" }
        }
    };

    check_str(r, "yes!");
}

#[test]
pub fn optional_slots() {
    let no_other = mview! {
        SlotIf cond=true {
            slot:Then { "yay!" }
        }
    };

    check_str(no_other, "yay!");

    let no_fallback = mview! {
        div {
            SlotIf cond=false {
                slot:Then { "not here" }
                slot:ElseIf cond=false { "not this either" }
            }
        }
    };

    check_str(no_fallback, "></div>")
}

#[component]
fn ChildThenIf(
    #[prop(into)] cond: MaybeSignal<bool>,
    children: ChildrenFn,
    #[prop(default=vec![])] else_if: Vec<ElseIf>,
    #[prop(optional)] fallback: Option<Fallback>,
) -> impl IntoView {
    move || {
        if cond() {
            (children)().into_view()
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
fn children_and_slots() {
    let then = mview! {
        ChildThenIf cond=true {
            "here"
            slot:ElseIf cond=true { "not :(" }
        }
    };

    check_str(
        then,
        Contains::AllOfNoneOf([["here"].as_slice(), ["not :("].as_slice()]),
    );

    let elseif = mview! {
        div {
            ChildThenIf cond=false {
                "not :("
                slot:ElseIf cond=true { "yes!" }
            }
        }
    };

    check_str(
        elseif,
        Contains::AllOfNoneOf([["yes!"].as_slice(), ["not :("].as_slice()]),
    );

    let mixed = mview! {
        div {
            ChildThenIf cond=true {
                "here 1"
                slot:ElseIf cond=false { "not this" }
                "here 2"
                span { "here 3" }
                slot:ElseIf cond=true { "still not here" }

                ChildThenIf cond=false {
                    "nested not here"
                    slot:Fallback { "nested is here!" }
                }

                slot:Fallback { "this one is not present" }
                "yet another shown"
            }
        }
    };

    check_str(
        mixed,
        Contains::AllOfNoneOf([
            [
                "here 1",
                "here 2",
                "<span data-hk=\"0-0-0-12\">here 3</span>",
                "nested is here!",
                "yet another shown",
            ]
            .as_slice(),
            [
                "not this",
                "still not here",
                "nested not here",
                "this one is not present",
            ]
            .as_slice(),
        ]),
    );
}

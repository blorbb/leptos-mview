use leptos::*;
use leptos_mview::mview;

#[slot]
struct Nothing {}

#[component]
fn TakesNothing(nothing: Nothing) -> impl IntoView { let _ = nothing; }

fn try_bad_dirs() {
    let attrs: Vec<(&'static str, Attribute)> = Vec::new();
    let _spread = mview! {
        TakesNothing {
            slot:Nothing {..attrs};
        }
    };

    let _on = mview! {
        TakesNothing {
            slot:Nothing on:click={|_| ()};
        }
    };

    let _attr = mview! {
        TakesNothing {
            slot:Nothing attr:something="something";
        }
    };

    fn a_directive(_el: HtmlElement<html::AnyElement>) {}
    let _use = mview! {
        TakesNothing {
            slot:Nothing use:a_directive;
        }
    };

    let _prop = mview! {
        TakesNothing {
            slot:Nothing prop:value="1";
        }
    };
}

fn main() {}

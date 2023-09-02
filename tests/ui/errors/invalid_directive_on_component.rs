use leptos::*;
use leptos_mview::view;

fn style() {
    view! {
        Component style:color="white";
    }
}

fn class() {
    view! {
        Component class:red={true};
    }
}

#[component]
fn Component() -> impl IntoView {
    view! {
        button;
    }
}

fn main() {}

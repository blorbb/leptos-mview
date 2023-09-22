use leptos::*;
use leptos_mview::view;

fn style_on_component() {
    view! {
        Component style:color="white";
    }
}

fn class_on_component() {
    view! {
        Component class:red={true};
    }
}

fn prop_on_component() {
    view! {
        Component prop:value="1";
    }
}

#[component]
fn SpreadOnComponent() -> impl IntoView {
    #[allow(unused_variables)]
    let attrs = vec![
        ("class", "something"),
        ("data", "a"),
    ];
    view! {
        Component {..attrs};
    }
}

fn attr_on_element() {
    view! {
        input attr:class="no" type="text";
    }
}

fn clone_on_element() {
    let notcopy = String::new();
    view! {
        div {
            span clone:{notcopy} {
                {notcopy.clone()}
            }
        }
    }
}

fn sel_shorthand_on_components() {
    view! {
        Component.not-working #some-id;
    }
}

#[component]
fn Component() -> impl IntoView {
    view! {
        button;
    }
}

fn main() {}

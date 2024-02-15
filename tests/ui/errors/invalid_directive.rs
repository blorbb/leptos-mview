use leptos::*;
use leptos_mview::mview;

fn not_directive() {
    mview! {
        div something:yes="b" {}
    };
}

fn not_class_name() {
    mview! {
        div class:("abcd") = true {}
    };
}

fn not_style_name() {
    mview! {
        div style:[1, 2]="black" {}
    };
}

fn not_event_name() {
    mview! {
        button on:clicky-click={move |_| ()};
    };
}

fn invalid_modifier() {
    mview! {
        button on:click:delegated={|_| ()};
    };
}

#[component]
fn Com(#[prop(optional, into)] class: TextProp) -> impl IntoView {
    let _ = class;
}

fn invalid_parts() {
    _ = mview! {
        div class:this:undelegated=true;
    };
    _ = mview! {
        div style:position:undelegated="absolute";
    };
    _ = mview! {
        input prop:value:something="input something";
    };
    _ = mview! {
        button use:directive:another;
    };
    _ = mview! {
        button attr:type="submit";
    };

    let to_clone = String::new();
    _ = mview! {
        Com clone:to_clone:undelegated;
    };
    _ = mview! {
        Com clone:{to_clone};
    };
    _ = mview! {
        Com class:aaa:undelegated=[false];
    };
}

fn directive(_el: leptos::HtmlElement<leptos::html::AnyElement>) {}

fn main() {}

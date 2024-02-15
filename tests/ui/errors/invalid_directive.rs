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

fn main() {}

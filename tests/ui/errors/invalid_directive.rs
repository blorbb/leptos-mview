use leptos_mview::view;

fn not_directive() {
    view! {
        div something:yes="b" {}
    };
}

fn not_class_name() {
    view! {
        div class:("abcd") = true {}
    };
}

fn not_style_name() {
    view! {
        div style:[1, 2]="black" {}
    };
}

fn not_event_name() {
    view! {
        button on:clicky-click={move |_| ()};
    };
}

fn main() {}

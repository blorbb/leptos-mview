use leptos_mview::mview;

fn unwrapped() {
    _ = mview! {
        div a=a {}
    };
}

fn no_spread() {
    _ = mview! {
        div {..};
    };
}

fn main() {}

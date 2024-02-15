use leptos_mview::mview;

fn invalid_value() {
    _ = mview! {
        div class:x={true} {
            span class=test
        }
    }
}

fn incomplete_directive() {
    _ = mview! {
        div class:x={true} {
            span class:
        }
    }
}

fn main() {}

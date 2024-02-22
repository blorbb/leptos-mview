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

// ensure that it is spanned to the delims, not call site
fn empty_value() {
    _ = mview! {
        a href={};
        a href=();
        a href=[];
    };
}

fn missing_value_no_remaining() {
    // nothing after the =, make sure that the error is on the = not call site
    _ = mview! {
        a href=
    };
}

fn main() {}

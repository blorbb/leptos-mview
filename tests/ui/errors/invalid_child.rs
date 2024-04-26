use leptos_mview::mview;

fn main() {
    let a = "a";
    mview! {
        (a)
    };
}

// checking that it doesn't suggest adding `|_| {value}`
fn not_impl_intoview() {
    // &&str doesn't impl IntoView, &str does
    // should be `{*value}`
    let value: &&str = &"hi";
    _ = mview! {
        span (
            {value}
        )
    };

    // forgot to call `.collect_view()`
    let values: Vec<&'static str> = vec!["hi", "bye", "howdy", "hello", "hey"];
    _ = mview! {
        ul {
            {values
                .into_iter()
                .map(|val: &str| {
                    mview! { li({val}) }
                })
            }
        }
    }
}

fn extra_semicolons() {
    _ = mview! {
        div { "hi there" };
        span;
    };
}

fn unreachable_code() {
    _ = mview! {
        div {
            {todo!()}
        }
    }
}

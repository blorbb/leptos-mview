use leptos_mview::view;

fn main() {
    _ = view! {
        div {
            "something"
            input.input type="text"
        }
    };
    compile_error!("test warnings");
}

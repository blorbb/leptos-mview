#![deny(unused_braces)]

use leptos_mview::mview;

fn main() {
    _ = mview! {
        div a={3} b={"aaaaa"} {
            {1234}
            span class={"braces not needed"} { "hi" }
        }
    };

    _ = mview! {
        button class:primary-200={true};
        button on:click={move |_| println!("hi")} {
            span 
                style:background-color={"black"}
                style:color="white"
            {
                "inverted"
            }
        }
    };
}

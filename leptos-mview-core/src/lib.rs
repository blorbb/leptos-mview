pub mod attribute;
pub mod children;
pub mod element;
pub mod ident;
pub mod value;

use element::Element;
use proc_macro2::TokenStream;
use quote::ToTokens;

pub fn component(input: TokenStream) -> TokenStream {
    eprintln!("starting macro");
    let element = match syn::parse2::<Element>(input) {
        Ok(tree) => tree,
        Err(e) => return e.to_compile_error(),
    };
    eprintln!("parsed macro successfully macro");
    eprintln!("{element:?}");
    element.into_token_stream()
}

pub mod attribute;
pub mod children;
pub mod element;
pub mod ident;
pub mod tag;
pub mod value;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::children::Children;

pub fn component(input: TokenStream) -> TokenStream {
    let fragment = match syn::parse2::<Children>(input) {
        Ok(tree) => tree,
        Err(e) => return e.to_compile_error(),
    };
    // If there's a single top level component, can just expand like
    // div().attr(...).child(...)...
    // If there are multiple top-level children, need to use the fragment.
    if fragment.len() == 1 {
        fragment
            .into_vec()
            .into_iter()
            .next()
            .expect("length should be 1")
            .into_token_stream()
    } else {
        fragment.to_fragment()
    }
}

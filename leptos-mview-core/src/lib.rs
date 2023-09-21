#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::module_name_repetitions
)]

mod attribute;
mod children;
mod element;
mod error_ext;
mod expand;
mod ident;
mod kw;
mod span;
mod tag;
mod value;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{children::Children, expand::children_fragment_tokens};

#[must_use]
pub fn component(input: TokenStream) -> TokenStream {
    let children = match syn::parse2::<Children>(input) {
        Ok(tree) => tree,
        Err(e) => return e.to_compile_error(),
    };
    // If there's a single top level component, can just expand like
    // div().attr(...).child(...)...
    // If there are multiple top-level children, need to use the fragment.
    if children.len() == 1 {
        let child = children.into_vec().remove(0);
        quote! {
            {
                #[allow(unused_braces)]
                #child
            }
        }
    } else {
        let fragment = children_fragment_tokens(&children);
        quote! {
            {
                #[allow(unused_braces)]
                #fragment
            }
        }
    }
}

#[warn(clippy::pedantic, clippy::nursery)]
mod attribute;
mod children;
mod element;
mod error_ext;
mod ident;
mod kw;
mod tag;
mod value;

use proc_macro2::TokenStream;
use quote::quote;

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
        let child = fragment.into_vec().remove(0);
        quote! {
            {
                #[allow(unused_braces)]
                #child
            }
        }
    } else {
        let fragment = fragment.to_fragment();
        quote! {
            {
                #[allow(unused_braces)]
                #fragment
            }
        }
    }
}

#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::module_name_repetitions
)]

mod ast;
mod error_ext;
mod expand;
mod kw;
mod parse;
mod span;

use ast::Child;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;

use crate::{ast::Children, expand::children_fragment_tokens};

#[must_use]
pub fn mview_impl(input: TokenStream) -> TokenStream {
    // return () in case of any errors, to avoid "unexpected end of macro
    // invocation" e.g. when assigning `let res = mview! { ... };`
    proc_macro_error::set_dummy(quote! { () });

    let children = match syn::parse2::<Children>(input) {
        Ok(tree) => tree,
        Err(e) => return e.to_compile_error(),
    };
    // If there's a single top level component, can just expand like
    // div().attr(...).child(...)...
    // If there are multiple top-level children, need to use the fragment.
    if children.len() == 1 {
        let child = children.into_vec().remove(0);
        match child {
            Child::Node(node) => quote! {
                { #[allow(unused_braces)] #node }
            },
            Child::Slot(slot, _) => abort!(
                slot.span(),
                "slots should be inside a parent that supports slots"
            ),
        }
    } else {
        // look for any slots
        if let Some(slot) = children.slot_children().next() {
            abort!(
                slot.tag().span(),
                "slots should be inside a parent that supports slots"
            );
        };

        let fragment = children_fragment_tokens(children.element_children());
        quote! {
            {
                #[allow(unused_braces)]
                #fragment
            }
        }
    }
}

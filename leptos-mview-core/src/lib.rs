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

use ast::{Child, Children};
use expand::root_children_tokens;
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::abort;
use quote::quote;
use syn::spanned::Spanned;

#[must_use]
pub fn mview_impl(input: TokenStream) -> TokenStream {
    // return () in case of any errors, to avoid "unexpected end of macro
    // invocation" e.g. when assigning `let res = mview! { ... };`
    proc_macro_error2::set_dummy(quote! { () });

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

        let fragment = root_children_tokens(children.element_children(), Span::call_site());
        quote! {
            {
                #[allow(unused_braces)]
                #fragment
            }
        }
    }
}

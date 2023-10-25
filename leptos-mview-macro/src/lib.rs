use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
#[rustfmt::skip]
pub fn mview(input: TokenStream) -> TokenStream {
    leptos_mview_core::mview_impl(input.into()).into()
}

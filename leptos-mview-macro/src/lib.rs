use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    leptos_mview_core::component(input.into()).into()
}

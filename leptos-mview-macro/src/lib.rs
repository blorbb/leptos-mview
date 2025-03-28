use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

/// A concise view macro for Leptos.
///
/// See [module documentation](https://docs.rs/leptos-mview/) for more usage details.
///
/// # Examples
///
/// ```
/// # use leptos_mview_macro::mview; use leptos::prelude::*;
/// let input = RwSignal::new(String::new());
/// let (red, set_red) = signal(true);
///
/// mview! {
///     !DOCTYPE html;
///     h1.title("A great website")
/// 
///     input
///         #some-id
///         type="text"
///         bind:value={input}
///         class:{red} // class:red={red}
///         on:click={move |_| set_red(false)};
/// 
///     // {move || !input().is_empty()}
///     Show when=[!input().is_empty()] (
///         Await
///             future={fetch_from_db(input())}
///             blocking
///         |db_info| (
///             em("DB info is: " {*db_info})
///             // {move || format!("{:?}", input())}
///             span("Query was: " f["{:?}", input()])
///         )
///     )
/// 
///     SlotIf cond={red} (
///         slot:Then("red")
///         slot:ElseIf cond={Signal::derive(move || input().is_empty())} ("empty")
///         slot:Fallback ("odd")
///     )
/// }
/// # ;
/// 
/// async fn fetch_from_db(input: String) -> usize { input.len() }
///
/// # #[slot] struct Then { children: ChildrenFn }
/// # #[slot] struct ElseIf { #[prop(into)] cond: Signal<bool>, children: ChildrenFn }
/// # #[slot] struct Fallback { children: ChildrenFn }
/// #
/// # #[component]
/// # fn SlotIf(
/// #     #[prop(into)] cond: Signal<bool>,
/// #     then: Then,
/// #     #[prop(optional)] else_if: Vec<ElseIf>,
/// #     #[prop(optional)] fallback: Option<Fallback>,
/// # ) -> impl IntoView {
/// #     move || {
/// #         if cond() {
/// #             (then.children)().into_any()
/// #         } else if let Some(else_if) = else_if.iter().find(|i| (i.cond)()) {
/// #             (else_if.children)().into_any()
/// #         } else if let Some(fallback) = &fallback {
/// #             (fallback.children)().into_any()
/// #         } else {
/// #             ().into_any()
/// #         }
/// #     }
/// # }
/// ```
#[proc_macro_error]
#[proc_macro]
#[rustfmt::skip]
pub fn mview(input: TokenStream) -> TokenStream {
    leptos_mview_core::mview_impl(input.into()).into()
}

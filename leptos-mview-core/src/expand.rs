//! Miscellaneous functions to convert structs to `TokenStream`s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;

use crate::{
    attribute::{directive::DirectiveKind, SimpleAttr},
    children::Children,
    element::Element,
    tag::Tag,
};

/// Converts an xml (like html, svg or math) element to tokens.
///
/// Returns `None` if the element is not an xml element (custom component).
///
/// # Example
/// ```ignore
/// use leptos::*;
/// let div = create_node_ref::<html::Div>();
/// view! {
///     div
///         class="component"
///         style:color="black"
///         ref={div}
///     {
///         "Hello " strong { "world" }
///     }
/// }
/// ```
/// Expands to:
/// ```ignore
/// div()
///     .attr("class", "component")
///     .style("color", "black")
///     .node_ref(div)
///     .child("Hello ")
///     .child(strong().child("world"))
/// ```
pub fn xml_to_tokens(element: &Element) -> Option<TokenStream> {
    let tag_path = match element.tag() {
        Tag::Component(_) => return None,
        Tag::Html(ident) => quote! { ::leptos::html::#ident() },
        Tag::Svg(ident) => quote! { ::leptos::svg::#ident() },
        Tag::Unknown(ident) => quote! { ::leptos::math::#ident() },
    };

    // parse normal attributes first
    let mut attrs = TokenStream::new();
    // put directives at the end so conditional attributes like `class:` work.
    let mut directives = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            SimpleAttr::Kv(attr) => {
                let key = attr.key();
                let value = attr.value();
                // special cases
                attrs.extend(if key.repr() == "ref" {
                    quote! { .node_ref(#value) }
                } else {
                    quote! { .attr(#key, #value) }
                });
            }
            SimpleAttr::Directive(attr) => {
                let dir = attr.directive();
                let name = attr.name();
                let name_ident = name.to_snake_ident();
                let value = attr.value();
                directives.extend(match attr.kind() {
                    DirectiveKind::Style | DirectiveKind::Class => quote! { .#dir(#name, #value) },
                    DirectiveKind::On => quote! { .#dir(::leptos::ev::#name_ident, #value) },
                    DirectiveKind::Clone => abort!(
                        dir.span(),
                        "directive `{}:` is not supported on html elements",
                        dir
                    ),
                });
            }
        }
    }

    let children = element.children().map(child_methods_tokens);

    Some(quote! {
        #tag_path
            #attrs
            #directives
            #children
    })
}

/// Transforms a component into a `TokenStream` of a leptos component view.
///
/// Returns `None` if `self.tag` is not a `Component`.
///
/// Example builder expansion of a component:
/// ```ignore
/// leptos::component_view(
///     &Com,
///     leptos::component_props_builder(&Com)
///         .num(3)
///         .text("a".to_string())
///         .children(Box::new(move || {
///             Fragment::lazy(|| [
///                 "child",
///                 "child2",
///             ])
///         }))
///         .build()
/// )
/// ```
///
/// Where the component has signature:
///
/// ```ignore
/// #[component]
/// pub fn Com(num: u32, text: String, children: Children) -> impl IntoView { ... }
/// ```
pub fn component_to_tokens(element: &Element) -> Option<TokenStream> {
    let Tag::Component(ident) = element.tag() else {
        return None;
    };

    // attribute methods to add when building
    let mut attrs = TokenStream::new();
    // the variables (idents) to clone before making children
    // in the form `let value = name.clone()`
    let mut clones = TokenStream::new();
    let mut event_listeners = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            SimpleAttr::Kv(attr) => {
                let key = attr.key().to_snake_ident();
                let value = attr.value();
                attrs.extend(quote! { .#key(#value) });
            }
            SimpleAttr::Directive(dir) => match dir.directive().kind() {
                DirectiveKind::On => {
                    let event = dir.name();
                    let callback = dir.value();
                    event_listeners.extend(quote! {
                        .on(
                            ::leptos::ev::undelegated(::leptos::ev::#event),
                            #callback
                        )
                    });
                }
                DirectiveKind::Clone => {
                    let to_clone = dir.name().to_snake_ident();
                    // value must just be an ident.
                    let Some(new_ident) = dir.value().as_block_with_ident() else {
                        abort!(
                            dir.value().span(),
                            "value of a `clone:` directive must be an ident like {}",
                            to_clone
                        );
                    };

                    clones.extend(quote! { let #new_ident = #to_clone.clone(); });
                }
                DirectiveKind::Class | DirectiveKind::Style => abort!(
                    dir.span(),
                    "directive `{}:` is not supported on components",
                    dir.directive()
                ),
            },
        }
    }

    // .children takes a boxed fragment
    let children = element
        .children()
        .map(children_fragment_tokens)
        .map(|fragment| {
            quote! {
                .children({
                    #clones
                    ::std::boxed::Box::new(move || {
                        #fragment
                    })
                })
            }
        });

    Some(quote! {
        ::leptos::component_view(
            &#ident,
            ::leptos::component_props_builder(&#ident)
                #attrs
                #children
                .build()
        )
        .into_view()
        #event_listeners
    })
}

/// Converts the children to a series of `.child` calls.
///
/// # Example
/// ```ignore
/// div { "a" {var} "b" }
/// ```
/// Expands to:
/// ```ignore
/// div().child("a").child({var}).child("b")
/// ```
pub fn child_methods_tokens(children: &Children) -> TokenStream {
    let children = children.iter();
    quote! {
        #( .child(#children) )*
    }
}

/// Converts the children into a `leptos::Fragment::lazy()` token stream.
///
/// Example:
/// ```ignore
/// "a"
/// {var}
/// "b"
/// ```
///
/// Should expand to:
/// ```ignore
/// Fragment::lazy(|| {
///     [
///         {"a"}.into_view(),
///         {var}.into_view(),
///         {"b"}.into_view(),
///     ].to_vec()
/// )
/// ```
pub fn children_fragment_tokens(children: &Children) -> TokenStream {
    let children = children.iter();
    quote! {
        ::leptos::Fragment::lazy(|| {
            [#(  ::leptos::IntoView::into_view(#children) ),*]
            .to_vec()
        })
    }
}

//! Miscellaneous functions to convert structs to `TokenStream`s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;

use crate::{
    attribute::{
        directive::{DirectiveAttr, DirectiveKind},
        kv::KvAttr,
        SimpleAttr,
    },
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
    let attrs = element.attrs().kv_attrs().map(|kv| xml_attr_tokens(kv));

    // put directives at the end so conditional attributes like `class:` work.
    let directives = element.attrs().directives().map(|dir| xml_directive_tokens(dir));

    let children = element.children().map(child_methods_tokens);

    Some(quote! {
        #tag_path
            #(#attrs)*
            #(#directives)*
            #children
    })
}

/// Expands an attribute to a `.attr(key, value)` token stream.
///
/// Some special attributes are converted differently, like `ref`.
///
/// # Example
/// ```ignore
/// div data-index=1 class="b" ref={div_ref};
/// ```
/// Expands to:
/// ```ignore
/// div()
///     .attr("data-index", 1)
///     .attr("class", "b")
///     .node_ref(div_ref)
/// ```
fn xml_attr_tokens(attr: &KvAttr) -> TokenStream {
    let key = attr.key();
    let value = attr.value();
    if key.repr() == "ref" {
        quote! { .node_ref(#value) }
    } else {
        quote! { .attr(#key, #value) }
    }
}

/// Converts a directive to a `.dir(name, value)` token stream.
///
/// # Example
/// ```ignore
/// div
///     class:some-thing=true
///     style:color="red"
///     on:click={handle_click};
/// ```
/// Expands to:
/// ```ignore
/// div()
///     .class("some-thing", true)
///     .style("color", "red")
///     .on(leptos::ev::click, {handle_click});
/// ```
fn xml_directive_tokens(attr: &DirectiveAttr) -> TokenStream {
    let dir = attr.directive();
    let name = attr.name();
    let name_ident = name.to_snake_ident();
    let value = attr.value();
    match attr.kind() {
        DirectiveKind::Style | DirectiveKind::Class => quote! { .#dir(#name, #value) },
        DirectiveKind::On => quote! { .#dir(::leptos::ev::#name_ident, #value) },
    }
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

    let attrs = element.attrs().iter().map(|a| match a {
        SimpleAttr::Kv(kv) => component_attr_tokens(kv),
        SimpleAttr::Directive(dir) => component_directive_tokens(dir),
    });

    // .children takes a boxed fragment
    let children = element
        .children()
        .map(children_fragment_tokens)
        .map(|tokens| {
            quote! {
                .children(
                    ::std::boxed::Box::new(move || #tokens)
                )
            }
        });

    Some(quote! {
        ::leptos::component_view(
            &#ident,
            ::leptos::component_props_builder(&#ident)
                #(#attrs)*
                #children
                .build()
        )
    })
}

/// Converts an attribute to a `.key(value)` token stream.
fn component_attr_tokens(attr: &KvAttr) -> TokenStream {
    let key = attr.key().to_snake_ident();
    let value = attr.value();
    quote! { .#key(#value) }
}

/// Converts an attribute to a `.key(value)` token stream.
///
/// Aborts if this directive is not supported on components. (Currently
/// only `on:` is supported)
fn component_directive_tokens(directive: &DirectiveAttr) -> TokenStream {
    match directive.directive().kind() {
        DirectiveKind::On => {
            let event = directive.name();
            let callback = directive.value();
            quote! {
                .on(
                    ::leptos::ev::undelegated(
                        ::leptos::ev::#event
                    ),
                    #callback
                )
            }
        }
        _ => abort!(
            directive.span(),
            "only `on:` directives are allowed on components"
        ),
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

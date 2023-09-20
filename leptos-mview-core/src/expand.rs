//! Miscellaneous functions to convert structs to `TokenStream`s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::token::CustomToken;

use crate::{
    attribute::{
        directive::{Directive, DirectiveAttr},
        selector::{SelectorShorthand, SelectorShorthands},
        Attr,
    },
    children::Children,
    element::Element,
    ident::KebabIdent,
    tag::Tag,
    value::Value,
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

    // add selector-style ids/classes (div.some-class #some-id)
    let selector_methods = xml_selectors_to_tokens(element.selectors());

    // parse normal attributes first
    let mut attrs = TokenStream::new();
    let mut spread_attrs = TokenStream::new();
    // put directives at the end so conditional attributes like `class:` work
    // with `class="..."` attributes
    let mut directives = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            Attr::Kv(attr) => {
                let key = attr.key();
                let value = attr.value();
                // special cases
                attrs.extend(if key.repr() == "ref" {
                    let node_ref = quote_spanned!(key.span()=> node_ref);
                    quote! { .#node_ref(#value) }
                } else {
                    quote! { .attr(#key, #value) }
                });
            }
            Attr::Directive(dir) => {
                fn method_key_value(dir: &impl Directive) -> TokenStream {
                    let (dir, name, value) = dir.explode();
                    quote! { .#dir(#name, #value) }
                }

                match dir {
                    DirectiveAttr::Class(c) => directives.extend(method_key_value(c)),
                    DirectiveAttr::Style(s) => directives.extend(method_key_value(s)),
                    DirectiveAttr::Prop(p) => {
                        let (dir, name, value) = p.explode();
                        let name_str = name.to_string();
                        let name = quote_spanned!(name.span()=> #name_str);
                        directives.extend(quote! { .#dir(#name, #value) })
                    },
                    DirectiveAttr::On(o) => {
                        let (dir, ev, value) = o.explode();
                        directives.extend(quote! { .#dir(::leptos::ev::#ev, #value) });
                    }
                    DirectiveAttr::Attr(a) => abort_not_supported(element, a),
                    DirectiveAttr::Clone(c) => abort_not_supported(element, c),
                }
            }
            Attr::Spread(spread) => {
                let ident = spread.as_ident();
                let method = quote_spanned!(ident.span()=> attrs);
                spread_attrs.extend(quote! {
                    .#method(#ident)
                });
            }
        }
    }

    let children = element.children().map(child_methods_tokens);

    Some(quote! {
        #tag_path
            #attrs
            #directives
            #selector_methods
            #spread_attrs
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
///         .children(::leptos::ToChildren::to_children(move || {
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

    // selectors not supported on components (for now)
    if !element.selectors().is_empty() {
        let first_prefix = element.selectors()[0].prefix();
        abort!(
            first_prefix.span(),
            "class/id selector shorthand is not allowed on components"
        );
    };

    // attribute methods to add when building
    let mut attrs = TokenStream::new();
    let mut dyn_attrs: Vec<(KebabIdent, Value)> = Vec::new();
    let mut first_dyn_attr_token = None;
    // the variables (idents) to clone before making children
    // in the form `let value = name.clone()`
    let mut clones = TokenStream::new();
    let mut event_listeners = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            Attr::Kv(attr) => {
                let key = attr.key().to_snake_ident();
                let value = attr.value();
                attrs.extend(quote! { .#key(#value) });
            }
            Attr::Spread(spread) => {
                abort!(
                    spread.as_ident().span(),
                    "spread attributes not supported on components"
                );
            }
            Attr::Directive(dir) => match dir {
                DirectiveAttr::On(o) => {
                    let (dir, ev, callback) = o.explode();
                    event_listeners.extend(quote! {
                        .#dir(
                            ::leptos::ev::undelegated(::leptos::ev::#ev),
                            #callback
                        )
                    });
                }
                DirectiveAttr::Attr(a) => {
                    let (dir, key, value) = a.explode();
                    dyn_attrs.push((key, value));
                    first_dyn_attr_token.get_or_insert(dir);
                }
                DirectiveAttr::Clone(c) => {
                    let (_, to_clone, cloned_ident) = c.explode();
                    let Some(cloned_ident) = cloned_ident.as_block_with_ident() else {
                        abort!(
                            cloned_ident.span(),
                            "value of a `clone:` directive must be an ident like {{{}}}",
                            to_clone
                        )
                    };
                    clones.extend(quote! { let #cloned_ident = #to_clone.clone(); });
                }
                DirectiveAttr::Class(c) => abort_not_supported(element, c),
                DirectiveAttr::Style(s) => abort_not_supported(element, s),
                DirectiveAttr::Prop(p) => abort_not_supported(element, p),
            },
        }
    }

    // children with arguments take a `Fn(T) -> impl IntoView`
    // normal children (`Children`, `ChildrenFn`, ...) take `ToChildren::to_children`
    let args = element.children_args();
    let children = element.children().map(|children| {
        let fragment = children_fragment_tokens(children);
        // only wrap the fragment if there are no closures
        let wrapped_fragment = if element.children_args().is_none() {
            quote! {
                ::leptos::ToChildren::to_children(move || #fragment)
            }
        } else {
            quote! { move |#args| #fragment }
        };

        quote! {
            .children({
                #clones
                #wrapped_fragment
            })
        }
    });

    // expand dyn attrs to the method if any exist
    let dyn_attrs = if dyn_attrs.is_empty() {
        None
    } else {
        let method = quote_spanned! {
            first_dyn_attr_token.unwrap().span=>
            dyn_attrs
        };
        let (names, values): (Vec<_>, Vec<_>) = dyn_attrs.into_iter().unzip();
        Some(quote! {
            .#method(
                <[_]>::into_vec(
                    ::std::boxed::Box::new([
                        #( (#names, ::leptos::IntoAttribute::into_attribute(#values)) ),*
                    ])
                )
            )
        })
    };

    Some(quote! {
        ::leptos::component_view(
            &#ident,
            ::leptos::component_props_builder(&#ident)
                #attrs
                #children
                .build()
                #dyn_attrs
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
            <[_]>::into_vec(
                ::std::boxed::Box::new([
                    #(  ::leptos::IntoView::into_view(#children) ),*
                ])
            )
        })
    }
}

/// Converts element class/id selector shorthands into a series of `.classes`
/// and `.id` calls.
fn xml_selectors_to_tokens(selectors: &SelectorShorthands) -> TokenStream {
    let (classes, ids): (Vec<_>, Vec<_>) = selectors
        .iter()
        .partition(|sel| matches!(sel, SelectorShorthand::Class { .. }));

    let classes_method = if classes.is_empty() {
        None
    } else {
        let method = quote_spanned!(classes[0].prefix().span()=> classes);
        let classes_str = classes
            .iter()
            .map(|class| class.ident().repr())
            .collect::<Vec<_>>()
            .join(" ");
        Some(quote! { .#method(#classes_str) })
    };

    let id_methods = ids.iter().map(|id| {
        let method = quote_spanned!(id.prefix().span()=> id);
        let ident = id.ident();
        quote!(.#method(#ident))
    });

    quote! { #classes_method #(#id_methods)* }
}

fn abort_not_supported<D: Directive>(element: &Element, dir: &D) -> ! {
    let suffix = match element.tag() {
        Tag::Html(_) => "html elements",
        Tag::Component(_) => "components",
        Tag::Svg(_) => "svgs",
        Tag::Unknown(_) => "web components",
    };
    abort!(
        dir.dir_key_span(),
        "directive {} is not supported on {}",
        D::Dir::display(),
        suffix
    )
}

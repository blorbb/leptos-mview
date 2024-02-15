//! Miscellaneous functions to convert structs to [`TokenStream`]s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};

use crate::ast::{
    attribute::{directive::Directive, selector::SelectorShorthand},
    Attr, Element, KebabIdent, KebabIdentOrStr, NodeChild, Tag,
};

/// Functions for specific parts of an element's expansion.
mod subroutines;
#[allow(clippy::wildcard_imports)]
use subroutines::*;
/// Small helper functions for converting types or emitting errors.
mod utils;
#[allow(clippy::wildcard_imports)]
use utils::*;

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
/// })
/// ```

// used in the root or for component children
pub fn children_fragment_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
    span: Span,
) -> TokenStream {
    quote_spanned! { span=>
        ::leptos::Fragment::lazy(|| {
            ::std::vec![
                #(  ::leptos::IntoView::into_view(#children) ),*
            ]
        })
    }
}

/// Converts an xml (like html, svg or math) element to tokens.
///
/// Returns `None` if the element is not an xml element (custom component).
///
/// # Example
/// ```ignore
/// use leptos::*;
/// use leptos_mview::mview;
/// let div = create_node_ref::<html::Div>();
/// mview! {
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
        Tag::Component(..) => return None,
        Tag::Html(ident) => quote! { ::leptos::html::#ident() },
        Tag::Svg(ident) => quote! { ::leptos::svg::#ident() },
        Tag::Math(ident) => quote! { ::leptos::math::#ident() },
        Tag::WebComponent(ident) => {
            let custom = syn::Ident::new("custom", ident.span());
            quote! { ::leptos::html::#custom(::leptos::html::Custom::new(#ident)) }
        }
    };

    // add selector-style ids/classes (div.some-class #some-id)
    let selector_methods = xml_selectors_tokens(element.selectors());

    // parse normal attributes first
    let mut attrs = TokenStream::new();
    let mut spread_attrs = TokenStream::new();
    // put directives at the end so conditional attributes like `class:` work
    // with `class="..."` attributes
    let mut directives = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            Attr::Kv(attr) => attrs.extend(xml_kv_attribute_tokens(attr)),
            Attr::Directive(dir) => directives.extend(xml_directive_tokens(dir)),
            Attr::Spread(spread) => spread_attrs.extend(xml_spread_tokens(spread)),
        }
    }

    let children = element
        .children()
        .map(|children| xml_child_methods_tokens(children.element_children()));

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
/// The const generic switches between parsing a slot and regular leptos
/// component, as the two implementations are very similar.
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
#[allow(clippy::too_many_lines)]
pub fn component_to_tokens<const IS_SLOT: bool>(element: &Element) -> Option<TokenStream> {
    let Tag::Component(ident, generics) = element.tag() else {
        return None;
    };

    // collect a bunch of info about the element attributes //

    // attribute methods to add when building
    let mut attrs = TokenStream::new();
    let mut dyn_attrs: Vec<&Directive> = Vec::new();
    let mut use_directives: Vec<&Directive> = Vec::new();
    // the variables (idents) to clone before making children
    // in the form `let name = name.clone();`
    let mut clones = TokenStream::new();
    let mut event_listeners = TokenStream::new();
    // components can take `.some-class` or `class:this={signal}` by passing it into
    // the `class` prop
    // .0 is the class string, .1 is the 'signal' (or `None` if using selectors)
    let mut dyn_classes: Vec<(KebabIdentOrStr, Option<TokenStream>)> = Vec::new();
    // ids are not reactive (no `id:this={signal}`), will just be from selectors
    let mut selector_ids: Vec<syn::LitStr> = Vec::new();

    for sel in element.selectors().iter() {
        match sel {
            SelectorShorthand::Id { id, .. } => selector_ids.push(id.to_lit_str()),
            SelectorShorthand::Class { class, .. } => {
                dyn_classes.push((KebabIdentOrStr::KebabIdent(class.clone()), None));
            }
        };
    }

    element.attrs().iter().for_each(|a| match a {
        Attr::Kv(attr) => attrs.extend(component_kv_attribute_tokens(attr)),
        Attr::Spread(spread) => {
            emit_error!(
                spread.span(),
                "spread attributes not supported on components/slots"
            );
        }
        Attr::Directive(dir) => match dir.dir.to_string().as_str() {
            "on" => {
                if IS_SLOT {
                    emit_error!(dir.dir.span(), "`on:` is not supported on slots");
                } else {
                    event_listeners.extend(event_listener_tokens(dir));
                }
            }
            "attr" => {
                if IS_SLOT {
                    emit_error!(dir.dir.span(), "`attr:` is not supported on slots");
                } else {
                    emit_error_if_modifier(dir.modifier.as_ref());
                    dyn_attrs.push(dir);
                }
            }
            "use" => {
                if IS_SLOT {
                    emit_error!(dir.dir.span(), "`use:` is not supported on slots");
                } else {
                    emit_error_if_modifier(dir.modifier.as_ref());
                    use_directives.push(dir);
                }
            }
            "clone" => {
                emit_error_if_modifier(dir.modifier.as_ref());
                clones.extend(component_clone_tokens(dir));
            }
            "class" => {
                emit_error_if_modifier(dir.modifier.as_ref());
                dyn_classes.push((dir.key.clone(), Some(dir.value.to_token_stream())));
            }
            "style" | "prop" => {
                emit_error!(
                    dir.dir.span(),
                    "`{}:` is not supported on components/slots",
                    dir.dir
                );
            }
            _ => {
                emit_error!(dir.dir.span(), "unknown directive");
            }
        },
    });

    // convert the collected info into tokens //

    let children = element.children().map(|children| {
        let mut it = children.element_children().peekable();
        // need to check that there are any element children at all,
        // as components that accept slots may not accept children.
        it.peek()
            .is_some()
            .then(|| component_children_tokens(it, element.children_args(), &clones))
    });

    let slot_children = element
        .children()
        .map(|children| slots_to_tokens(children.slot_children()));

    let dyn_attrs = component_dyn_attrs_to_methods(&dyn_attrs);
    let use_directives = use_directives.into_iter().map(use_directive_to_method);
    let dyn_classes = component_classes_to_method(dyn_classes);
    let selector_ids = component_ids_to_method(selector_ids);

    // if attributes are missing, an error is made in `.build()` by the component
    // builder.
    let build = quote_spanned!(ident.span()=> .build());

    if IS_SLOT {
        // Into is for turning a single slot into a vec![slot] if needed
        Some(quote! {
            ::std::convert::Into::into(
                #ident #generics::builder()
                    #attrs
                    #dyn_classes
                    #selector_ids
                    #children
                    #build
            )
        })
    } else {
        Some(quote! {
            // the .build() returns `!` if not all props are present.
            // this causes unreachable code warning in ::leptos::component_view
            #[allow(unreachable_code)]
            ::leptos::component_view(
                &#ident,
                ::leptos::component_props_builder(&#ident #generics)
                    #attrs
                    #dyn_classes
                    #selector_ids
                    #children
                    #slot_children
                    #build
                    #dyn_attrs
            )
            .into_view()
            #(#use_directives)*
            #event_listeners
        })
    }
}

#[allow(clippy::doc_markdown)]
/// Converts a list of slots to a bunch of methods to be called on the parent
/// component.
///
/// The iterator must have only elements that are slots.
///
/// Slots are expanded from:
/// ```ignore
/// Tabs {
///     slot:Tab label="tab1" { "content" }
/// }
/// ```
/// to:
/// ```ignore
/// leptos::component_props_builder(&Tabs)
///     .tab(vec![
///         Tab::builder()
///             .label("tab1")
///             .children( /* expansion of "content" to a component child */ )
///             .build()
///             .into()
///     ])
/// ```
/// Where the slot's name is converted to snake_case for the method name.
fn slots_to_tokens<'a>(children: impl Iterator<Item = &'a Element>) -> TokenStream {
    // collect to hashmap //

    // Mapping from the slot name (component, UpperCamelCase name, not snake_case)
    // to a vec of the each slot's expansion.
    let mut slot_children = HashMap::<KebabIdent, Vec<TokenStream>>::new();
    for el in children {
        let component_name = el.tag().ident();

        let slot_component =
            component_to_tokens::<true>(el).expect("all children should be slot components");
        slot_children
            .entry(component_name)
            .or_default()
            .push(slot_component);
    }

    // convert to tokens //
    slot_children
        .into_iter()
        .map(|(slot_name, slot_tokens)| {
            let method = syn::Ident::new_raw(
                &utils::upper_camel_to_snake_case(slot_name.repr()),
                slot_name.span(),
            );

            if slot_tokens.len() == 1 {
                // don't wrap in a vec
                quote! {
                    .#method(#(#slot_tokens)*)
                }
            } else {
                quote! {
                    .#method(::std::vec![
                        #(#slot_tokens),*
                    ])
                }
            }
        })
        .collect()
}

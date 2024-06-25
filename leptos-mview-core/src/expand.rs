//! Miscellaneous functions to convert structs to [`TokenStream`]s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

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
            <[_]>::into_vec(::std::boxed::Box::new([
                #(  ::leptos::IntoView::into_view(#children) ),*
            ]))
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
            let ident = ident.to_lit_str();
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
    let Tag::Component(path) = element.tag() else {
        return None;
    };
    let path = turbofishify(path.clone());

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
    let mut classes_span: Option<Span> = None;
    // ids are not reactive (no `id:this={signal}`), will just be from selectors
    let mut selector_ids: Vec<KebabIdent> = Vec::new();
    let mut id_span: Option<Span> = None;

    let mut spread_attrs = TokenStream::new();

    for sel in element.selectors().iter() {
        match sel {
            SelectorShorthand::Id { id, pound_symbol } => {
                selector_ids.push(id.clone());
                id_span.get_or_insert(pound_symbol.span);
            }
            SelectorShorthand::Class { class, dot_symbol } => {
                dyn_classes.push((KebabIdentOrStr::KebabIdent(class.clone()), None));
                classes_span.get_or_insert(dot_symbol.span);
            }
        };
    }

    element.attrs().iter().for_each(|a| match a {
        Attr::Kv(attr) => attrs.extend(component_kv_attribute_tokens(attr)),
        Attr::Spread(spread) => {
            if IS_SLOT {
                emit_error!(spread.span(), "spread syntax is not supported on slots");
            } else {
                spread_attrs.extend(component_spread_tokens(spread));
            }
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
                classes_span.get_or_insert(dir.dir.span());
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
    let dyn_classes = component_classes_to_method(dyn_classes, classes_span);
    let selector_ids = component_ids_to_method(selector_ids, id_span);

    // if attributes are missing, an error is made in `.build()` by the component
    // builder.
    let build = quote_spanned!(path.span()=> .build());

    if IS_SLOT {
        // Into is for turning a single slot into a vec![slot] if needed
        Some(quote! {
            ::std::convert::Into::into(
                #path::builder()
                    #attrs
                    #dyn_classes
                    #selector_ids
                    #children
                    #build
            )
        })
    } else {
        // this whole thing needs to be spanned to avoid errors occurring at the whole
        // call site.
        let component_props_builder = quote_spanned! {
            path.span()=> ::leptos::component_props_builder(&#path)
        };

        Some(quote! {
            // the .build() returns `!` if not all props are present.
            // this causes unreachable code warning in ::leptos::component_view
            #[allow(unreachable_code)]
            ::leptos::IntoView::into_view(
                ::leptos::component_view(
                    &#path,
                    #component_props_builder
                        #attrs
                        #dyn_classes
                        #selector_ids
                        #children
                        #slot_children
                        #build
                        #dyn_attrs
                        #spread_attrs
                )
            )
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
    let mut slot_children = HashMap::<syn::Ident, Vec<TokenStream>>::new();
    for el in children {
        let Tag::Component(path) = el.tag() else {
            panic!("called `slots_to_tokens` on non-slot element")
        };
        let slot_name = if let Some(ident) = path.get_ident() {
            ident.clone()
        } else {
            emit_error!(path.span(), "slot name must be a single ident, not a path");
            continue;
        };

        let slot_component =
            component_to_tokens::<true>(el).expect("checked that element is a component");
        slot_children
            .entry(slot_name)
            .or_default()
            .push(slot_component);
    }

    // convert to tokens //
    slot_children
        .into_iter()
        .map(|(slot_name, slot_tokens)| {
            let method = syn::Ident::new_raw(
                &utils::upper_camel_to_snake_case(&slot_name.to_string()),
                slot_name.span(),
            );

            if slot_tokens.len() == 1 {
                // don't wrap in a vec
                quote! {
                    .#method(#(#slot_tokens)*)
                }
            } else {
                quote! {
                    .#method(<[_]>::into_vec(::std::boxed::Box::new([
                        #(#slot_tokens),*
                    ])))
                }
            }
        })
        .collect()
}

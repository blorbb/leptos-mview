//! Miscellaneous functions to convert structs to [`TokenStream`]s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use proc_macro_error2::emit_error;
use quote::{quote, quote_spanned};
use syn::{ext::IdentExt, parse_quote, parse_quote_spanned, spanned::Spanned};

use crate::ast::{
    attribute::{directive::Directive, selector::SelectorShorthand},
    Attr, Element, KebabIdent, KebabIdentOrStr, NodeChild, Tag, Value,
};

/// Functions for specific parts of an element's expansion.
mod subroutines;
#[allow(clippy::wildcard_imports)]
use subroutines::*;
/// Small helper functions for converting types or emitting errors.
mod utils;
#[allow(clippy::wildcard_imports)]
use utils::*;

/// Converts the children into a `View::new()` token stream.
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
/// View::new((
///     {"a"},
///     {var},
///     {"b"},
/// ))
/// ```
pub fn root_children_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
    span: Span,
) -> TokenStream {
    quote_spanned! { span=>
        ::leptos::prelude::View::new((
            #( #children, )*
        ))
    }
}

// used for component children
pub fn children_fragment_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
    span: Span,
) -> TokenStream {
    let children = children.collect::<Vec<_>>();
    let has_multiple_children = children.len() > 1;

    if has_multiple_children {
        quote_spanned! { span=>
            ( #( #children, )* )
        }
    } else {
        quote_spanned! { span=>
            #( #children )*
        }
    }
}

/// Converts an xml (like html, svg or math) element to tokens.
///
/// Returns `None` if the element is not an xml element (custom component).
///
/// # Example
/// ```ignore
/// use leptos::prelude::*;
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
///     .class("component")
///     .style(("color", "black"))
///     .node_ref(div)
///     .child(IntoRender::into_render("Hello "))
///     .child(IntoRender::into_render(strong().child("world")))
/// ```
pub fn xml_to_tokens(element: &Element) -> Option<TokenStream> {
    let tag_path = match element.tag() {
        Tag::Component(..) => return None,
        Tag::Html(ident) => quote! { ::leptos::tachys::html::element::#ident() },
        Tag::Svg(ident) => quote! { ::leptos::tachys::svg::element::#ident() },
        Tag::Math(ident) => quote! { ::leptos::tachys::math::element::#ident() },
        Tag::WebComponent(ident) => {
            let ident = ident.to_lit_str();
            let custom = syn::Ident::new("custom", ident.span());
            quote! { ::leptos::tachys::html::element::#custom(#ident) }
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
            Attr::Kv(attr) => attrs.extend(xml_kv_attribute_tokens(attr, element.tag().kind())),
            Attr::Directive(dir) => directives.extend(xml_directive_tokens(dir)),
            Attr::Spread(spread) => spread_attrs.extend(xml_spread_tokens(spread)),
        }
    }

    let children = element
        .children()
        .map(|children| xml_child_methods_tokens(children.node_children()));

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
    let mut directive_paths: Vec<TokenStream> = Vec::new();
    // the variables (idents) to clone before making children
    // in the form `let name = name.clone();`
    let mut clones = TokenStream::new();

    // shorthands are not supported on slots
    if IS_SLOT {
        if let Some(first) = element.selectors().first() {
            emit_error!(
                first.prefix(),
                "selector shorthands are not supported on slots"
            )
        }
    } else {
        // all the ids need to be collected together
        // as multiple attr:id=... creates multiple `id=...` attributes on teh element
        let mut ids = Vec::<KebabIdent>::new();
        let mut first_pound_symbol = None;
        for sel in element.selectors().iter() {
            match sel {
                SelectorShorthand::Id { id, pound_symbol } => {
                    first_pound_symbol.get_or_insert(*pound_symbol);
                    ids.push(id.clone());
                }
                SelectorShorthand::Class { class, dot_symbol } => {
                    // desugar to class:the-class
                    directive_paths.push(
                        directive_to_any_attr_path(&Directive {
                            dir: syn::Ident::new("class", dot_symbol.span),
                            key: KebabIdentOrStr::KebabIdent(class.clone()),
                            modifier: None,
                            value: None,
                        })
                        .expect("class directive is known"),
                    );
                }
            };
        }
        // push all the ids as directive
        if let Some(first_pound_symbol) = first_pound_symbol {
            let joined_ids = ids
                .iter()
                .map(|ident| ident.repr())
                .collect::<Vec<_>>()
                .join(" ");
            // desugar to attr:id="the-id id2 id3"
            directive_paths.push(
                directive_to_any_attr_path(&Directive {
                    dir: syn::Ident::new("attr", Span::call_site()),
                    key: parse_quote_spanned! { first_pound_symbol.span=> id },
                    modifier: None,
                    value: Some(Value::Lit(parse_quote!(#joined_ids))),
                })
                .expect("attr directive is known"),
            );
        }
    }

    element.attrs().iter().for_each(|a| match a {
        Attr::Kv(attr) => attrs.extend(component_kv_attribute_tokens(attr)),
        Attr::Spread(spread) => {
            if IS_SLOT {
                emit_error!(spread.span(), "spread syntax is not supported on slots");
            } else {
                directive_paths.push(component_spread_tokens(spread));
            }
        }
        Attr::Directive(dir) => match dir.dir.to_string().as_str() {
            // clone works on both components and slots
            "clone" => {
                emit_error_if_modifier(dir.modifier.as_ref());
                clones.extend(component_clone_tokens(dir));
            }
            // slots support no other directives
            other if IS_SLOT => {
                emit_error!(dir.dir.span(), "`{}:` is not supported on slots", other);
            }
            _ => {
                if let Some(path) = directive_to_any_attr_path(dir) {
                    directive_paths.push(path);
                } else {
                    emit_error!(dir.dir.span(), "unknown directive");
                }
            }
        },
    });

    // convert the collected info into tokens //

    let children = element.children().map(|children| {
        let mut it = children.node_children().peekable();
        // need to check that there are any element children at all,
        // as components that accept slots may not accept children.
        it.peek()
            .is_some()
            .then(|| component_children_tokens(it, element.children_args(), &clones))
    });

    let slot_children = element
        .children()
        .map(|children| slots_to_tokens(children.slot_children()));

    // if attributes are missing, an error is made in `.build()` by the component
    // builder.
    let build = quote_spanned!(path.span()=> .build());

    if IS_SLOT {
        // Into is for turning a single slot into a vec![slot] if needed
        Some(quote! {
            ::std::convert::Into::into(
                #path::builder()
                    #attrs
                    #children
                    #build
            )
        })
    } else {
        // this whole thing needs to be spanned to avoid errors occurring at the whole
        // call site.
        let component_props_builder = quote_spanned! {
            path.span()=> ::leptos::component::component_props_builder(&#path)
        };

        let directive_paths = (!directive_paths.is_empty()).then(|| {
            quote! {
                .add_any_attr((#(#directive_paths,)*))
            }
        });

        Some(quote! {
            ::leptos::component::component_view(
                &#path,
                #component_props_builder
                    #attrs
                    #children
                    #slot_children
                    #build
            )
            #directive_paths
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
                &utils::upper_camel_to_snake_case(&slot_name.unraw().to_string()),
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

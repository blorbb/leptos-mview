//! Miscellaneous functions to convert structs to [`TokenStream`]s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use crate::ast::{
    attribute::{
        directive::{self, DirectiveAttr},
        kv::KvAttr,
        selector::{SelectorShorthand, SelectorShorthands},
        spread_attrs::SpreadAttr,
    },
    Attr, Element, KebabIdent, NodeChild, Tag,
};

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
        .map(|children| child_methods_tokens(children.element_children()));

    Some(quote! {
        #tag_path
            #attrs
            #directives
            #selector_methods
            #spread_attrs
            #children
    })
}

/// Converts element class/id selector shorthands into a series of `.classes`
/// and `.id` calls.
fn xml_selectors_tokens(selectors: &SelectorShorthands) -> TokenStream {
    let (classes, ids): (Vec<_>, Vec<_>) = selectors
        .iter()
        .partition(|sel| matches!(sel, SelectorShorthand::Class { .. }));

    let classes_method = if classes.is_empty() {
        None
    } else {
        let method = syn::Ident::new("classes", classes[0].prefix().span());
        let classes_str = classes
            .iter()
            .map(|class| class.ident().repr())
            .collect::<Vec<_>>()
            .join(" ");
        Some(quote! { .#method(#classes_str) })
    };

    let id_methods = ids.iter().map(|id| {
        let method = proc_macro2::Ident::new("id", id.prefix().span());
        let ident = id.ident();
        quote!(.#method(#ident))
    });

    quote! { #classes_method #(#id_methods)* }
}

fn xml_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let key = attr.key();
    let value = attr.value();
    // special cases
    if key.repr() == "ref" {
        let node_ref = syn::Ident::new("node_ref", key.span());
        quote! { .#node_ref(#value) }
    } else {
        quote! { .attr(#key, #value) }
    }
}

fn xml_directive_tokens(directive: &DirectiveAttr) -> TokenStream {
    match directive {
        DirectiveAttr::Class(c) => {
            let (dir, name, value) = c.explode();
            quote! { .#dir(#name, #value) }
        }
        DirectiveAttr::Style(s) => {
            let (dir, name, value) = s.explode();
            quote! { .#dir(#name, #value) }
        }
        DirectiveAttr::Prop(p) => {
            let (dir, name, value) = p.explode();
            let name_str = name.to_string();
            let name = quote_spanned!(name.span()=> #name_str);
            quote! { .#dir(#name, #value) }
        }
        DirectiveAttr::On(o) => {
            let (dir, ev, value) = o.explode();
            quote! { .#dir(::leptos::ev::#ev, #value) }
        }
        DirectiveAttr::Use(u) => use_directive_to_method(u),
        DirectiveAttr::Attr(a) => abort!(a.full_span(), "`attr:` not supported on elements"),
        DirectiveAttr::Clone(c) => abort!(c.full_span(), "`clone:` not supported on elements"),
    }
}

fn xml_spread_tokens(attr: &SpreadAttr) -> TokenStream {
    let (dotdot, expr) = (attr.dotdot(), attr.expr());
    let attrs = syn::Ident::new("attrs", dotdot.span());
    quote! {
        .#attrs(#expr)
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
pub fn child_methods_tokens<'a>(children: impl Iterator<Item = &'a NodeChild>) -> TokenStream {
    quote! {
        #( .child(#children) )*
    }
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
    let mut dyn_attrs: Vec<&directive::Attr> = Vec::new();
    let mut use_directives: Vec<&directive::Use> = Vec::new();
    // the variables (idents) to clone before making children
    // in the form `let name = name.clone();`
    let mut clones = TokenStream::new();
    let mut event_listeners = TokenStream::new();
    // components can take `.some-class` or `class:this={signal}` by passing it into
    // the `class` prop
    // .0 is the class string, .1 is the 'signal' (or just "move || true" if using
    // selectors)
    let mut dyn_classes: Vec<(syn::LitStr, TokenStream)> = Vec::new();
    // ids are not reactive (no `id:this={signal}`), will just be from selectors
    let mut selector_ids: Vec<syn::LitStr> = Vec::new();

    for sel in element.selectors().iter() {
        match sel {
            SelectorShorthand::Id { id, .. } => selector_ids.push(id.to_lit_str()),
            SelectorShorthand::Class { dot_symbol, class } => dyn_classes.push((
                class.to_lit_str(),
                quote_spanned!(dot_symbol.span=> move || true),
            )),
        };
    }

    element.attrs().iter().for_each(|a| match a {
        Attr::Kv(attr) => attrs.extend(component_kv_attribute_tokens(attr)),
        Attr::Spread(spread) => {
            abort!(
                spread.span(),
                "spread attributes not supported on components"
            );
        }
        Attr::Directive(dir) => match dir {
            DirectiveAttr::On(o) => {
                if IS_SLOT {
                    abort!(o.full_span(), "`on:` not supported on slots");
                } else {
                    event_listeners.extend(component_event_listener_tokens(o));
                }
            }
            // TODO: seems like attr: could be supported on slots, but #[prop(attrs)] isn't
            // supported. allow them if they are updated in the future.
            DirectiveAttr::Attr(a) => {
                if IS_SLOT {
                    abort!(a.full_span(), "`attr:` not supported on slots");
                } else {
                    dyn_attrs.push(a);
                }
            }
            DirectiveAttr::Clone(c) => clones.extend(component_clone_tokens(c)),
            DirectiveAttr::Use(u) => {
                if IS_SLOT {
                    abort!(u.full_span(), "`use:` not supported on slots");
                } else {
                    use_directives.push(u);
                }
            }
            DirectiveAttr::Class(c) => {
                dyn_classes.push((c.key().clone(), c.value().to_token_stream()));
            }
            DirectiveAttr::Style(s) => {
                abort!(s.full_span(), "`style:` not supported on components/slots");
            }
            DirectiveAttr::Prop(p) => {
                abort!(p.full_span(), "`prop:` not supported on components/slots");
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

    let dyn_attrs = dyn_attrs_to_methods(&dyn_attrs);
    let use_directives = use_directives.into_iter().map(use_directive_to_method);
    let dyn_classes = component_classes_to_method(dyn_classes);
    let selector_ids = component_ids_to_method(selector_ids);

    // if attributes are missing, an error is made in `.build()` by the component
    // builder.
    let build = quote_spanned!(ident.span()=> .build());

    if IS_SLOT {
        // `unreachable_code` warning is generated at Into
        // Into is for turning a single slot into a vec![slot] if needed
        let into = quote_spanned!(ident.span()=> ::std::convert::Into::into);
        Some(quote! {
            #into(
                #ident #generics::builder()
                    #attrs
                    #dyn_classes
                    #selector_ids
                    #children
                    #build
            )
        })
    } else {
        // `unreachable_code` warning is generated in both of these
        let component_view = quote_spanned!(ident.span()=> ::leptos::component_view);
        let component_props_builder =
            quote_spanned!(ident.span()=> ::leptos::component_props_builder);

        Some(quote! {
            #component_view(
                &#ident,
                #component_props_builder(&#ident #generics)
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

fn component_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let (key, value) = (attr.key().to_snake_ident(), attr.value());
    quote_spanned! {attr.span()=> .#key(#value) }
}

fn component_event_listener_tokens(dir: &directive::On) -> TokenStream {
    let (dir, ev, callback) = dir.explode();
    quote! {
        .#dir(
            ::leptos::ev::undelegated(::leptos::ev::#ev),
            #callback
        )
    }
}

/// Expands to a `let` statement `let to_clone = to_clone.clone();`.
fn component_clone_tokens(dir: &directive::Clone) -> TokenStream {
    let to_clone = dir.key();
    quote! { let #to_clone = #to_clone.clone(); }
}

/// Converts children to tokens for use by components.
///
/// The expansion is generally:
///
/// If there are no closure arguments,
/// ```ignore
/// .children({
///     // any clones
///     let clone = clone.clone();
///     // the children themself
///     leptos::ToChildren::to_children(move || {
///         leptos::Fragment::lazy(|| {
///             [
///                 child1.into_view(),
///                 child2.into_view(),
///             ].to_vec()
///         })
///     })
/// })
/// ```
///
/// If there are closure arguments,
/// ```ignore
/// .children({
///     // any clones
///     let clone = clone.clone();
///     // the children
///     move |args| leptos::Fragment::lazy(|| {
///         [
///             child1.into_view(),
///             child2.into_view(),
///         ].to_vec()
///     })
/// })
/// ```
fn component_children_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
    args: Option<&TokenStream>,
    clones: &TokenStream,
) -> TokenStream {
    let mut children = children.peekable();
    let child_span = children
        .peek()
        // not sure why `child.span()` is calling `syn::spanned::Spanned` instead
        .map_or_else(Span::call_site, |child| (*child).span());

    let children_fragment =
        children_fragment_tokens(children, args.map_or(child_span, Spanned::span));

    // children with arguments take a `Fn(T) -> impl IntoView`
    // normal children (`Children`, `ChildrenFn`, ...) take
    // `ToChildren::to_children`
    let wrapped_fragment = if let Some(args) = args {
        // `args` includes the pipes
        quote_spanned!(args.span()=> move #args #children_fragment)
    } else {
        // this span is required for slots that take `Callback<T, View>` but have been
        // given a regular `ChildrenFn` instead.
        let closure = quote_spanned!(child_span=> move || #children_fragment);
        quote! {
            ::leptos::ToChildren::to_children(#closure)
        }
    };

    let children_method = quote_spanned!(child_span=> children);

    quote! {
        .#children_method({
            #clones
            #wrapped_fragment
        })
    }
}

fn dyn_attrs_to_methods(dyn_attrs: &[&directive::Attr]) -> Option<TokenStream> {
    // expand dyn attrs to the method if any exist
    if dyn_attrs.is_empty() {
        return None;
    };

    let dyn_attrs_method = syn::Ident::new("dyn_attrs", dyn_attrs[0].dir().span);

    let (keys, values): (Vec<_>, Vec<_>) = dyn_attrs.iter().map(|a| (a.key(), a.value())).unzip();
    Some(quote! {
        .#dyn_attrs_method(
            ::std::vec![
                #( (#keys, ::leptos::IntoAttribute::into_attribute(#values)) ),*
            ]
        )
    })
}

// special attributes on components that add to a special set of props //

/// Adds potentially reactive classes to the `class` attribute of a component.
///
/// If no classes are reactive, a static string will be passed in. Otherwise,
/// the string is constructed and updated at runtime, which may have performance
/// drawbacks as the entire prop is updated if one signal changes.
///
/// The intended use is as follows:
/// ```ignore
/// // TODO: use prop(optional) when Default added to TextProp
/// #[component]
/// fn TakesClasses(#[prop(into, default="".into())] class: TextProp) -> impl IntoView {}
///
/// let signal = RwSignal::new(true);
///
/// mview! {
///     TakesClasses.class-1.another-class class:reactive={signal};
/// }
/// ```
///
/// For now, what is passed in to `{signal}` must be something that impls `Fn()
/// -> bool`, it cannot just be a `bool`.
fn component_classes_to_method(classes: Vec<(syn::LitStr, TokenStream)>) -> Option<TokenStream> {
    if classes.is_empty() {
        return None;
    };

    let first_span = classes[0].0.span();

    // if there are no reactive classes, just create the string now
    // add `||` to reject `class:thing={true}`
    if classes
        .iter()
        .all(|(_, signal)| signal.to_string().ends_with("|| true"))
    {
        let string = classes
            .into_iter()
            .map(|(class, _)| class.value())
            .collect::<Vec<_>>()
            .join(" ");
        Some(quote_spanned!(first_span=> .class(#string)))
    } else {
        // there are reactive classes: need to construct it at runtime

        // TODO: is there a way to accept both `bool` and `Fn() -> bool`?
        // maybe `leptos::Class`?

        let classes_array = classes.into_iter().map(|(class, signal)| {
            // add extra bracket to make sure the closure is called
            let signal_called = quote_spanned! { signal.span()=> (#signal)() };
            // use fully qualified path so that error says 'incorrect type' instead of
            // 'method `then_some` not found'
            quote_spanned! { signal_called.span()=>
                ::std::primitive::bool::then_some(#signal_called, #class)
            }
        });
        let classes_array = quote_spanned!(first_span=> [#(#classes_array),*]);
        let contents = quote_spanned! { first_span=>
            #classes_array
                .iter()
                .flatten() // remove None
                .cloned() // turn &&str to &str
                .collect::<Vec<&str>>()
                .join(" ")
        };

        // span to the first item
        Some(quote_spanned! { first_span=>
            .class(move || #contents)
        })
    }
}

/// Adds a list of strings to the `id` prop of a component.
///
/// IDs should not be changed reactively, so it is not supported.
fn component_ids_to_method(ids: Vec<syn::LitStr>) -> Option<TokenStream> {
    if ids.is_empty() {
        return None;
    };

    let first_span = ids[0].span();
    // ids are not reactive, so just give one big string
    let ids = ids
        .into_iter()
        .map(|id| id.value())
        .collect::<Vec<_>>()
        .join(" ");

    Some(quote_spanned!(first_span=> .id(#ids)))
}

/// Converts a `use:directive={value}` to a method.
///
/// The expansion for components and xml elements are the same.
///
/// ```text
/// use:d => .directive(d, ())
/// use:d={some_value} => .directive(d, some_value)
/// ```
fn use_directive_to_method(u: &directive::Use) -> TokenStream {
    let (use_token, func, value) = u.explode();
    let directive = syn::Ident::new("directive", use_token.span);
    let value = value.as_ref().map_or(
        quote_spanned! {func.span()=> () },
        ToTokens::to_token_stream,
    );
    quote! { .#directive(#func, #value) }
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
/// })
/// ```
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
                &upper_camel_to_snake_case(slot_name.repr()),
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

#[allow(clippy::doc_markdown)]
// just doing a manual implementation as theres only one need for this (slots).
// Use the `paste` crate if more are needed in the future.
/// `ident` must be an UpperCamelCase word with only ascii word characters.
fn upper_camel_to_snake_case(ident: &str) -> String {
    let mut new = String::with_capacity(ident.len());
    // all characters should be ascii
    for char in ident.chars() {
        // skip the first `_`.
        if char.is_ascii_uppercase() && !new.is_empty() {
            new.push('_');
        };
        new.push(char.to_ascii_lowercase());
    }

    new
}

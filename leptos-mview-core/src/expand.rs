//! Miscellaneous functions to convert structs to `TokenStream`s.

// putting specific `-> TokenStream` implementations here to have it all
// grouped instead of scattered throughout struct impls.

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::token::CustomToken;

use crate::{
    attribute::{
        directive::{self, Directive, DirectiveAttr},
        kv::KvAttr,
        selector::{SelectorShorthand, SelectorShorthands},
        spread_attrs::SpreadAttr,
        Attr,
    },
    children::Children,
    element::{ClosureArgs, Element},
    span,
    tag::{Tag, TagKind},
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
        Tag::Component(..) => return None,
        Tag::Html(ident) => quote! { ::leptos::html::#ident() },
        Tag::Svg(ident) => quote! { ::leptos::svg::#ident() },
        Tag::Math(ident) => quote! { ::leptos::math::#ident() },
        Tag::Unknown(ident) => {
            let custom = quote_spanned!(ident.span()=> custom);
            quote! { ::leptos::html::#custom(::leptos::html::Custom::new(#ident)) }
        }
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
            Attr::Kv(attr) => attrs.extend(xml_kv_attribute_tokens(attr)),
            Attr::Directive(dir) => directives.extend(xml_directive_tokens(element, dir)),
            Attr::Spread(spread) => spread_attrs.extend(xml_spread_tokens(spread)),
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

fn xml_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let key = attr.key();
    let value = attr.value();
    // special cases
    if key.repr() == "ref" {
        let node_ref = quote_spanned!(key.span()=> node_ref);
        quote! { .#node_ref(#value) }
    } else {
        quote! { .attr(#key, #value) }
    }
}

fn xml_directive_tokens(element: &Element, directive: &DirectiveAttr) -> TokenStream {
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
        DirectiveAttr::Attr(a) => abort_not_supported(&element.tag().kind(), a),
        DirectiveAttr::Clone(c) => abort_not_supported(&element.tag().kind(), c),
    }
}

fn xml_spread_tokens(attr: &SpreadAttr) -> TokenStream {
    let ident = attr.as_ident();
    let method = quote_spanned!(ident.span()=> attrs);
    quote! {
        .#method(#ident)
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
    let Tag::Component(ident, generics) = element.tag() else {
        return None;
    };

    // selectors not supported on components (for now)
    if !element.selectors().is_empty() {
        let first_prefix = element.selectors()[0].prefix();
        let last_ident = element.selectors().last().unwrap().ident();
        abort!(
            span::join(first_prefix.span(), last_ident.span()),
            "class/id selector shorthand is not supported on components"
        );
    };

    // attribute methods to add when building
    let mut attrs = TokenStream::new();
    let mut dyn_attrs: Vec<&directive::Attr> = Vec::new();
    // the variables (idents) to clone before making children
    // in the form `let value = name.clone()`
    let mut clones = TokenStream::new();
    let mut event_listeners = TokenStream::new();

    for a in element.attrs().iter() {
        match a {
            Attr::Kv(attr) => attrs.extend(component_kv_attribute_tokens(attr)),
            Attr::Spread(spread) => {
                abort!(
                    spread.span(),
                    "spread attributes not supported on components"
                );
            }
            Attr::Directive(dir) => match dir {
                DirectiveAttr::On(o) => event_listeners.extend(component_event_listener_tokens(o)),
                DirectiveAttr::Attr(a) => dyn_attrs.push(a),
                DirectiveAttr::Clone(c) => clones.extend(component_clone_tokens(c)),
                DirectiveAttr::Class(c) => abort_not_supported(&element.tag().kind(), c),
                DirectiveAttr::Style(s) => abort_not_supported(&element.tag().kind(), s),
                DirectiveAttr::Prop(p) => abort_not_supported(&element.tag().kind(), p),
            },
        }
    }

    let children = element
        .children()
        .map(|children| component_children_tokens(children, element.children_args(), &clones));

    let dyn_attrs = dyn_attrs_to_methods(&dyn_attrs);

    Some(quote! {
        ::leptos::component_view(
            &#ident,
            ::leptos::component_props_builder(&#ident #generics)
                #attrs
                #children
                .build()
                #dyn_attrs
        )
        .into_view()
        #event_listeners
    })
}

fn component_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let (key, value) = (attr.key().to_snake_ident(), attr.value());
    quote! { .#key(#value) }
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

/// Aborts if the directive value is not a block with an ident.
fn component_clone_tokens(dir: &directive::Clone) -> TokenStream {
    let (_, to_clone, cloned_ident) = dir.explode();
    let Some(cloned_ident) = cloned_ident.as_block_with_ident() else {
        abort!(
            cloned_ident.span(),
            "value of a `clone:` directive must be an ident like `{{{}}}`",
            to_clone
        )
    };
    quote! { let #cloned_ident = #to_clone.clone(); }
}

fn component_children_tokens(
    children: &Children,
    args: Option<&ClosureArgs>,
    clones: &TokenStream,
) -> TokenStream {
    let children_fragment = children_fragment_tokens(children);

    // children with arguments take a `Fn(T) -> impl IntoView`
    // normal children (`Children`, `ChildrenFn`, ...) take `ToChildren::to_children`
    let wrapped_fragment = if args.is_none() {
        quote! {
            ::leptos::ToChildren::to_children(move || #children_fragment)
        }
    } else {
        quote! { move |#args| #children_fragment }
    };

    quote! {
        .children({
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

    let dyn_attrs_method = quote_spanned! {
        dyn_attrs[0].dir().span=>
        dyn_attrs
    };

    let (keys, values): (Vec<_>, Vec<_>) = dyn_attrs.iter().map(|a| (a.key(), a.value())).unzip();
    Some(quote! {
        .#dyn_attrs_method(
            <[_]>::into_vec(
                ::std::boxed::Box::new([
                    #( (#keys, ::leptos::IntoAttribute::into_attribute(#values)) ),*
                ])
            )
        )
    })
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

/// Aborts with an appropriate message when a directive is not supported.
fn abort_not_supported<D: Directive>(tag: &TagKind, dir: &D) -> ! {
    let suffix = match tag {
        TagKind::Html => "html elements",
        TagKind::Component => "components",
        TagKind::Svg => "svgs",
        TagKind::Math => "math elements",
        TagKind::Unknown => "web components",
    };
    abort!(
        dir.full_span(),
        "directive {} is not supported on {}",
        D::Dir::display(),
        suffix
    )
}

use proc_macro2::{Span, TokenStream};
use proc_macro_error2::emit_error;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    ast::{
        attribute::{
            directive::Directive,
            kv::KvAttr,
            selector::{SelectorShorthand, SelectorShorthands},
            spread_attrs::SpreadAttr,
        },
        KebabIdent, KebabIdentOrStr, NodeChild,
    },
    expand::{children_fragment_tokens, emit_error_if_modifier},
    span,
};

////////////////////////////////////////////////////////////////
// ------------------- shared subroutines ------------------- //
////////////////////////////////////////////////////////////////

/// Converts a `use:directive={value}` to a method.
///
/// The expansion for components and xml elements are the same.
///
/// ```text
/// use:d => .directive(d, ().into())
/// use:d={some_value} => .directive(d, some_value.into())
/// ```
///
/// **Panics** if the provided directive is not `use:`.
pub(super) fn use_directive_to_method(u: &Directive) -> TokenStream {
    let Directive {
        dir: use_token,
        key,
        modifier,
        value,
    } = u;
    assert_eq!(use_token, "use", "directive should be `use:`");
    let directive_fn = key.to_ident_or_emit();
    emit_error_if_modifier(modifier.as_ref());

    let directive = syn::Ident::new("directive", use_token.span());
    let value = value.as_ref().map_or_else(
        || quote_spanned! {directive_fn.span()=> ().into() },
        |val| quote! { ::std::convert::Into::into(#val) },
    );
    quote! { .#directive(#directive_fn, #value) }
}

pub(super) fn event_listener_tokens(dir: &Directive) -> TokenStream {
    let Directive {
        dir,
        key,
        modifier,
        value,
    } = dir;
    assert_eq!(dir, "on", "directive should be `on:`");

    let ev_name = match key {
        KebabIdentOrStr::KebabIdent(ident) => ident.to_snake_ident(),
        KebabIdentOrStr::Str(s) => {
            emit_error!(s.span(), "event type must be an identifier");
            syn::Ident::new("invalid_event", s.span())
        }
    };

    let event = if let Some(modifier) = modifier {
        if modifier == "undelegated" {
            quote! { ::leptos::ev::#modifier(::leptos::ev::#ev_name) }
        } else {
            emit_error!(
                modifier.span(), "unknown modifier";
                help = ":undelegated is the only known modifier"
            );
            quote! { ::leptos::ev::#ev_name }
        }
    } else {
        quote! { ::leptos::ev::#ev_name }
    };
    quote! { .#dir(#event, #value) }
}

///////////////////////////////////////////////////////////
// ------------------- html/xml only ------------------- //
///////////////////////////////////////////////////////////

/// Converts element class/id selector shorthands into a series of `.classes`
/// and `.id` calls.
pub(super) fn xml_selectors_tokens(selectors: &SelectorShorthands) -> TokenStream {
    let (classes, ids): (Vec<_>, Vec<_>) = selectors
        .iter()
        .partition(|sel| matches!(sel, SelectorShorthand::Class { .. }));

    let class_methods = classes.iter().map(|class| {
        let method = syn::Ident::new("class", class.prefix().span());
        let class_name = class.ident().to_str_colored();
        quote! { .#method(#class_name, true) }
    });

    let id_methods = ids.iter().map(|id| {
        let method = syn::Ident::new("id", id.prefix().span());
        let id_name = id.ident().to_str_colored();
        quote! { .#method(#id_name) }
    });

    quote! { #(#class_methods)* #(#id_methods)* }
}

pub(super) fn xml_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let key = attr.key();
    let value = attr.value();
    // special cases
    if key.repr() == "ref" {
        let node_ref = syn::Ident::new("node_ref", key.span());
        quote! { .#node_ref(#value) }
    } else {
        // don't span the attribute to the string, unnecessary and makes it
        // string-colored
        let key = key.repr();
        quote! { .attr(#key, #value) }
    }
}

pub(super) fn xml_directive_tokens(directive: &Directive) -> TokenStream {
    let Directive {
        dir,
        key,
        modifier,
        value,
    } = directive;

    match dir.to_string().as_str() {
        "class" | "style" | "prop" => {
            let key = key.to_lit_str();
            emit_error_if_modifier(modifier.as_ref());
            quote! { .#dir(#key, #value) }
        }
        "on" => event_listener_tokens(directive),
        "use" => use_directive_to_method(directive),
        "attr" | "clone" => {
            emit_error!(dir.span(), "`{}:` is not supported on elements", dir);
            quote! {}
        }
        _ => {
            emit_error!(dir.span(), "unknown directive");
            quote! {}
        }
    }
}

pub(super) fn xml_spread_tokens(attr: &SpreadAttr) -> TokenStream {
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
pub(super) fn xml_child_methods_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
) -> TokenStream {
    let mut ts = TokenStream::new();
    for child in children {
        let child_method = syn::Ident::new("child", child.span());
        ts.extend(quote! {
            .#child_method(#child)
        });
    }
    ts
}

////////////////////////////////////////////////////////////
// ------------------- component only ------------------- //
////////////////////////////////////////////////////////////

pub(super) fn component_kv_attribute_tokens(attr: &KvAttr) -> TokenStream {
    let (key, value) = (attr.key().to_snake_ident(), attr.value());
    quote_spanned! { attr.span()=> .#key(#value) }
}

/// Expands to a `let` statement `let to_clone = to_clone.clone();`.
pub(super) fn component_clone_tokens(dir: &Directive) -> TokenStream {
    let to_clone = dir.key.to_ident_or_emit();
    emit_error_if_modifier(dir.modifier.as_ref());
    if let Some(value) = &dir.value {
        emit_error!(value.span(), "`clone:` does not take any values");
    };

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
pub(super) fn component_children_tokens<'a>(
    children: impl Iterator<Item = &'a NodeChild>,
    args: Option<&TokenStream>,
    clones: &TokenStream,
) -> TokenStream {
    let mut children = children.peekable();
    let child_span = children
        .peek()
        // not sure why `child.span()` is calling `syn::spanned::Spanned` instead
        .map_or_else(Span::call_site, |child| (*child).span());

    // span call site if there are no args so that the children don't get all the
    // `std` `vec!` etc docs.
    let children_fragment =
        children_fragment_tokens(children, args.map_or(Span::call_site(), Spanned::span));

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

pub(super) fn component_dyn_attrs_to_methods(dyn_attrs: &[&Directive]) -> Option<TokenStream> {
    // expand dyn attrs to the method if any exist
    if dyn_attrs.is_empty() {
        return None;
    };

    let dyn_attrs_method = syn::Ident::new("dyn_attrs", dyn_attrs[0].dir.span());

    let (keys, values): (Vec<_>, Vec<_>) = dyn_attrs
        .iter()
        .map(|a| (a.key.to_lit_str(), &a.value))
        .unzip();

    Some(quote! {
        .#dyn_attrs_method(
            <[_]>::into_vec(std::boxed::Box::new([
                #( (#keys, ::leptos::IntoAttribute::into_attribute(#values)) ),*
            ]))
        )
    })
}

pub(super) fn component_spread_tokens(attr: &SpreadAttr) -> TokenStream {
    let (dotdot, expr) = (attr.dotdot(), attr.expr());
    let dyn_bindings = syn::Ident::new("dyn_bindings", dotdot.span());
    quote! {
        .#dyn_bindings(#expr)
    }
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
/// #[component]
/// fn TakesClasses(#[prop(optional, into)] class: TextProp) -> impl IntoView {}
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
///
/// Returns [`None`] if `class_span` is [`None`] or `classes` is empty.
pub(super) fn component_classes_to_method(
    classes: Vec<(KebabIdentOrStr, Option<TokenStream>)>,
    class_span: Option<Span>,
) -> Option<TokenStream> {
    let Some(class_span) = class_span else { return None };
    if classes.is_empty() {
        return None;
    };

    fn generate_dummy_assignments(
        idents: impl IntoIterator<Item = (KebabIdentOrStr, Option<TokenStream>)>,
    ) -> impl Iterator<Item = TokenStream> {
        idents
            .into_iter()
            .filter_map(|(maybe_ident, _)| match maybe_ident {
                KebabIdentOrStr::KebabIdent(ident) => {
                    Some(span::color_all(ident.spans()).collect::<TokenStream>())
                }
                KebabIdentOrStr::Str(_) => None,
            })
    }

    // if there are no reactive classes, just create the string
    if classes.iter().all(|(_, signal)| signal.is_none()) {
        let string = classes
            .iter()
            .map(|(class, _)| class.to_lit_str().value())
            .collect::<Vec<_>>()
            .join(" ");

        let dummy_assignments = generate_dummy_assignments(classes);

        let class = quote_spanned!(class_span=> class);
        Some(quote!(.#class({
            #(#dummy_assignments)*
            #string
        })))
    } else {
        // there are reactive classes: need to construct it at runtime

        // TODO: is there a way to accept both `bool` and `Fn() -> bool`?
        // maybe `leptos::Class`?

        let classes_array = {
            let classes_iter = classes.iter().map(|(class, signal)| {
                let class_str = class.to_lit_str();
                let bool_signal = match signal {
                    Some(signal) => {
                        // add extra bracket to make sure the closure is called
                        quote_spanned!(signal.span()=> (#signal)())
                    }
                    None => quote!(true),
                };

                // use fully qualified path so that error says 'incorrect type' instead of
                // 'method `then_some` not found'
                quote_spanned! { bool_signal.span()=>
                    ::std::primitive::bool::then_some(#bool_signal, #class_str)
                }
            });

            quote_spanned!(class_span=> [#(#classes_iter),*])
        };
        let contents = quote_spanned! { class_span=>
            #classes_array
                .iter()
                .flatten() // remove None
                .cloned() // turn &&str to &str
                .collect::<Vec<&str>>()
                .join(" ")
        };

        let dummy_assignments = generate_dummy_assignments(classes);

        let class = quote_spanned!(class_span=> class);
        Some(quote! {
            .#class(move || {
                #(#dummy_assignments)*
                #contents
            })
        })
    }
}

/// Adds a list of strings to the `id` prop of a component.
///
/// IDs should not be changed reactively, so it is not supported.
///
/// Returns [`None`] if `id_span` is [`None`] or `ids` is empty.
pub(super) fn component_ids_to_method(
    ids: Vec<KebabIdent>,
    id_span: Option<Span>,
) -> Option<TokenStream> {
    let id_span = id_span?;
    if ids.is_empty() {
        return None;
    };

    // ids are not reactive, so just give one big string
    let id_str = ids
        .iter()
        .map(|id| id.to_lit_str().value())
        .collect::<Vec<_>>()
        .join(" ");

    let dummy_assignments = ids
        .into_iter()
        .map(|ident| span::color_all(ident.spans()).collect::<TokenStream>());

    let id = quote_spanned!(id_span=> id);
    Some(quote!(.#id({
        #(#dummy_assignments)*
        #id_str
    })))
}

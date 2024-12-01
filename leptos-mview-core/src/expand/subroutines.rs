use proc_macro2::{Span, TokenStream};
use proc_macro_error2::emit_error;
use quote::{quote, quote_spanned};
use syn::{ext::IdentExt, spanned::Spanned};

use crate::{
    ast::{
        attribute::{
            directive::Directive,
            kv::KvAttr,
            selector::{SelectorShorthand, SelectorShorthands},
            spread_attrs::SpreadAttr,
        },
        KebabIdentOrStr, NodeChild, TagKind, Value,
    },
    expand::{children_fragment_tokens, emit_error_if_modifier},
};

////////////////////////////////////////////////////////////////
// ------------------- shared subroutines ------------------- //
////////////////////////////////////////////////////////////////

/// Converts a `use:directive={value}` to a key (function) and value.
///
/// ```text
/// use:d => (d, ().into())
/// use:d={some_value} => (d, some_value.into())
/// ```
///
/// **Panics** if the provided directive is not `use:`.
pub(super) fn use_directive_fn_value(u: &Directive) -> (syn::Ident, TokenStream) {
    let Directive {
        dir: use_token,
        key,
        modifier,
        value,
    } = u;
    assert_eq!(use_token, "use", "directive should be `use:`");
    let directive_fn = key.to_ident_or_emit();
    emit_error_if_modifier(modifier.as_ref());

    let value = value.as_ref().map_or_else(
        || quote_spanned! {directive_fn.span()=> ().into() },
        |val| quote! { ::std::convert::Into::into(#val) },
    );
    (directive_fn, value)
}

pub(super) fn event_listener_event_path(dir: &Directive) -> TokenStream {
    let Directive {
        dir,
        key,
        modifier,
        value: _,
    } = dir;
    assert_eq!(dir, "on", "directive should be `on:`");

    let ev_name = match key {
        KebabIdentOrStr::KebabIdent(ident) => ident.to_snake_ident(),
        KebabIdentOrStr::Str(s) => {
            emit_error!(s.span(), "event type must be an identifier");
            syn::Ident::new("invalid_event", s.span())
        }
    };

    if let Some(modifier) = modifier {
        if modifier == "undelegated" {
            quote! {
                ::leptos::tachys::html::event::#modifier(
                    ::leptos::tachys::html::event::#ev_name
                )
            }
        } else {
            emit_error!(
                modifier.span(), "unknown modifier";
                help = ":undelegated is the only known modifier"
            );
            quote! { ::leptos::tachys::html::event::#ev_name }
        }
    } else {
        quote! { ::leptos::tachys::html::event::#ev_name }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttributeKind {
    /// "class"
    Class,
    /// "style"
    Style,
    /// An attribute with a `-`, like data-*
    ///
    /// Excludes `aria-*` attributes.
    Custom,
    /// An attribute that should be added by a method so that it is checked.
    OtherChecked,
}

impl AttributeKind {
    pub fn is_custom(self) -> bool { self == Self::Custom }

    pub fn is_class_or_style(self) -> bool { matches!(self, Self::Class | Self::Style) }
}

impl From<&str> for AttributeKind {
    fn from(value: &str) -> Self {
        if value == "class" {
            Self::Class
        } else if value == "style" {
            Self::Style
        } else if value.contains('-') && !value.starts_with("aria-") {
            Self::Custom
        } else {
            Self::OtherChecked
        }
    }
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
        quote! { .#method((#class_name, true)) }
    });

    let id_methods = ids.iter().map(|id| {
        let method = syn::Ident::new("id", id.prefix().span());
        let id_name = id.ident().to_str_colored();
        quote! { .#method(#id_name) }
    });

    quote! { #(#class_methods)* #(#id_methods)* }
}

pub(super) fn xml_kv_attribute_tokens(attr: &KvAttr, element_tag: TagKind) -> TokenStream {
    let key = attr.key();
    let value = attr.value();
    // special cases
    if key.repr() == "ref" {
        let node_ref = syn::Ident::new("node_ref", key.span());
        quote! { .#node_ref(#value) }
    } else {
        // https://github.com/leptos-rs/leptos/blob/main/leptos_macro/src/view/mod.rs#L960
        // Use unchecked attributes if:
        // - it's not `class` nor `style`, and
        // - It's a custom web component or SVG element
        // - or it's a custom or data attribute (has `-` except for `aria-`)
        let attr_kind = AttributeKind::from(key.repr());
        let is_web_or_svg = matches!(element_tag, TagKind::Svg | TagKind::WebComponent);

        if (is_web_or_svg || attr_kind.is_custom()) && !attr_kind.is_class_or_style() {
            // unchecked attribute
            // don't span the attribute to the string, unnecessary and makes it
            // string-colored
            let key = key.repr();
            quote! { .attr(#key, ::leptos::prelude::IntoAttributeValue::into_attribute_value(#value)) }
        } else {
            // checked attribute
            let key = key.to_snake_ident();
            quote! { .#key(#value) }
        }
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
        "class" | "style" => {
            let key = key.to_lit_str();
            emit_error_if_modifier(modifier.as_ref());
            quote! { .#dir((#key, #value)) }
        }
        "prop" => {
            let key = key.to_lit_str();
            emit_error_if_modifier(modifier.as_ref());
            quote! { .#dir(#key, #value) }
        }
        "on" => {
            let event_path = event_listener_event_path(directive);
            quote! { .#dir(#event_path, #value) }
        }
        "use" => {
            let (fn_name, value) = use_directive_fn_value(directive);
            let directive = syn::Ident::new("directive", dir.span());
            quote! {
                .#directive(#fn_name, #value)
            }
        }
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
    let attrs = syn::Ident::new("add_any_attr", dotdot.span());
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
            ::leptos::children::ToChildren::to_children(#closure)
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

// https://github.com/leptos-rs/leptos/blob/5947aa299e5299eb3dc75c58e28affb15e79b6ff/leptos_macro/src/view/mod.rs#L998

/// Converts a directive on a component to a path to be used on
/// `.add_any_attr(...)`
///
/// Returns [`None`] if the directive is an unknown directive, or `clone`.
///
/// Adding these directives to a component looks like:
/// ```ignore
/// View::new(
///     leptos::component::component_view(.., ..)
///     .add_any_attr((
///         leptos::tachys::html::class::class("something"),
///         leptos::tachys::html::class::class(("conditional", true)),
///         leptos::tachys::html::style::style(("position", "absolute")),
///         leptos::tachys::html::attribute::contenteditable(true),
///         leptos::tachys::html::attribute::custom::custom_attribute("data-index", 0),
///         leptos::tachys::html::property::prop("value", "aaaa"),
///         leptos::tachys::html::event::on(
///             leptos::tachys::html::event::undelegated(
///                 leptos::tachys::html::event::click
///             ),
///             || ()
///         ),
///         leptos::tachys::html::directive::directive(directive_name, ().into())
///     ))
/// )
/// ```
pub(super) fn directive_to_any_attr_path(directive: &Directive) -> Option<TokenStream> {
    let dir = &directive.dir;
    let path = match &*dir.to_string() {
        "class" | "style" => {
            // avoid making it string coloured
            let key = directive.key.to_unspanned_string();
            let value = directive.value.clone().unwrap_or_else(Value::new_true);
            // to avoid spanning the directive to the module
            let dir_unspanned = syn::Ident::new(&dir.to_string(), Span::call_site());
            quote! {
                ::leptos::tachys::html::#dir_unspanned::#dir((#key, #value))
            }
        }
        "attr" => {
            let attr_kind = AttributeKind::from(&*directive.key.to_lit_str().value());
            match attr_kind {
                AttributeKind::Class | AttributeKind::Style => {
                    let class_or_style = directive.key.to_ident_or_emit();
                    let value = directive.value.clone().unwrap_or_else(Value::new_true);
                    // to avoid spanning to the module name
                    let class_or_style_unspanned =
                        syn::Ident::new(&class_or_style.unraw().to_string(), Span::call_site());
                    quote! {
                        ::leptos::tachys::html::#class_or_style_unspanned::#class_or_style(#value)
                    }
                }
                AttributeKind::Custom => {
                    let attr_name = directive.key.to_unspanned_string();
                    let value = directive.value.clone().unwrap_or_else(Value::new_true);
                    quote! {
                        ::leptos::tachys::html::attribute::custom::custom_attribute(#attr_name, #value)
                    }
                }
                AttributeKind::OtherChecked => {
                    let attr_name = directive.key.to_ident_or_emit();
                    let value = directive.value.clone().unwrap_or_else(Value::new_true);
                    quote! {
                        ::leptos::tachys::html::attribute::#attr_name(#value)
                    }
                }
            }
        }
        "prop" => {
            let prop = directive.key.to_ident_or_emit();
            let value = directive.value.clone().unwrap_or_else(Value::new_true);
            quote! {
                ::leptos::tachys::html::property::#prop(#value)
            }
        }
        "on" => {
            let event_path = event_listener_event_path(directive);
            let value = &directive.value;
            quote! {
                ::leptos::tachys::html::event::on(#event_path, #value)
            }
        }
        "use" => {
            let (fn_name, value) = use_directive_fn_value(directive);
            let directive_method = syn::Ident::new("directive", directive.dir.span());
            quote! {
                ::leptos::tachys::html::directive::#directive_method(
                    #fn_name,
                    #value
                )
            }
        }
        _ => return None,
    };

    Some(path)
}

/// This should be added with all the other directives.
///
/// Spread attrs are added as `.add_any_attr(expr)`.
pub(super) fn component_spread_tokens(attr: &SpreadAttr) -> TokenStream { attr.expr().clone() }

use proc_macro_error::{abort, emit_error};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

use crate::{
    attribute::{selector::SelectorShorthands, SimpleAttrs},
    children::Children,
    error_ext::ResultExt,
    expand::{component_to_tokens, xml_to_tokens},
    tag::Tag,
};

type ClosureArgs = Punctuated<syn::Pat, Token![,]>;

/// A HTML or custom component.
///
/// Syntax mostly looks like this:
/// ```text
/// div class="blue" { "hello!" }
/// ^^^ ^^^^^^^^^^^^   ^^^^^^^^
/// tag attributes     children
/// ```
///
/// If the element ends in a semicolon, `children` is `None`.
/// ```text
/// input type="text";
/// br;
/// ```
#[derive(Debug)]
pub struct Element {
    tag: Tag,
    selectors: SelectorShorthands,
    attrs: SimpleAttrs,
    children_args: Option<ClosureArgs>,
    children: Option<Children>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse_children_block(input: ParseStream) -> syn::Result<Children> {
            let children;
            syn::braced!(children in input);
            children.parse::<Children>()
        }

        let tag: Tag = input.parse()?;
        let selectors: SelectorShorthands = input.parse()?;
        let attrs: SimpleAttrs = input.parse()?;

        if input.peek(Token![;]) {
            // no children, terminated by semicolon.
            input.parse::<Token![;]>().unwrap();
            Ok(Self::new(tag, selectors, attrs, None, None))
        } else if input.is_empty() {
            // allow no ending token if its the last child
            // makes for better editing experience when writing sequentially,
            // as syntax highlighting/autocomplete doesn't work if macro
            // can't fully compile.
            Ok(Self::new(tag, selectors, attrs, None, None))
        } else if input.peek(syn::token::Brace) {
            // has children in brace.
            let children = parse_children_block(input)?;
            Ok(Self::new(tag, selectors, attrs, None, Some(children)))
        } else if input.peek(Token![|]) {
            // maybe extra args for the children
            let args = parse_closure_args(input).unwrap_or_abort();
            // must have children block after
            if !input.peek(syn::token::Brace) {
                abort!(
                    input.span(),
                    "expected children block after closure arguments"
                )
            }
            let children = parse_children_block(input)?;
            Ok(Self::new(tag, selectors, attrs, Some(args), Some(children)))
        } else {
            // add error at the unknown token
            emit_error!(input.span(), "unknown attribute");
            abort!(
                tag.span().join(input.span()).unwrap_or(tag.span()), "child elements not found";
                note = "if you don't want any child elements, end the element with \
                a semi-colon `;` or empty braces `{}`."
            )
        }
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(xml_to_tokens(self).unwrap_or_else(|| component_to_tokens(self).unwrap()));
    }
}

impl Element {
    pub const fn new(
        tag: Tag,
        selectors: SelectorShorthands,
        attrs: SimpleAttrs,
        child_args: Option<ClosureArgs>,
        children: Option<Children>,
    ) -> Self {
        Self {
            tag,
            selectors,
            attrs,
            children_args: child_args,
            children,
        }
    }

    pub const fn tag(&self) -> &Tag {
        &self.tag
    }

    pub const fn selectors(&self) -> &SelectorShorthands {
        &self.selectors
    }

    pub const fn attrs(&self) -> &SimpleAttrs {
        &self.attrs
    }

    pub const fn children_args(&self) -> Option<&ClosureArgs> {
        self.children_args.as_ref()
    }

    pub const fn children(&self) -> Option<&Children> {
        self.children.as_ref()
    }
}

fn parse_closure_args(input: ParseStream) -> syn::Result<ClosureArgs> {
    input.parse::<Token![|]>()?;
    let mut args = Punctuated::new();
    loop {
        if input.peek(Token![|]) {
            break;
        }
        let value = syn::Pat::parse_single(input)?;
        args.push_value(value);
        if input.peek(Token![|]) {
            break;
        }
        let punct: Token![,] = input.parse()?;
        args.push_punct(punct);
    }
    input.parse::<Token![|]>().unwrap();
    Ok(args)
}

#[cfg(test)]
mod tests {

    use super::Element;

    #[test]
    fn full_element() {
        let input = r#"div class="test" checked data-index=[index] { "child" span { "child2" } }"#;
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 3);
        assert!(element.children().is_some());
        assert_eq!(element.children().unwrap().len(), 2);
    }

    #[test]
    fn no_child_element() {
        let input = r#"input type="text";"#;
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 1);
        assert!(element.children().is_none());
    }

    #[test]
    fn no_child_or_attrs() {
        let input = "br;";
        let element: Element = syn::parse_str(input).unwrap();
        assert_eq!(element.attrs().len(), 0);
        assert!(element.children.is_none());
    }
}

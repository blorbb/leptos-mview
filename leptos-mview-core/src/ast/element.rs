use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error::{abort, emit_error};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::{
    attribute::{
        selector::{SelectorShorthand, SelectorShorthands},
        Attr,
    },
    Attrs, Children, Tag,
};
use crate::{
    error_ext::ResultExt,
    expand::{component_to_tokens, xml_to_tokens},
    parse, span,
};

/// A HTML or custom component.
///
/// Consists of:
/// 1. [`tag`](Tag): The HTML/SVG/MathML element name, or leptos component name.
/// 2. [`selectors`](SelectorShorthands): Shortcut ways of writing `class="..."`
///    or `id="..."`. A list of classes or ids prefixed with a `.` or `#`
///    respectively.
/// 3. [`attrs`](Attrs): A space-separated list of attributes.
/// 4. [`children_args`](TokenStream): Optional arguments for the children,
///    placed in closure pipes `|...|` immediately before the children block.
///    The closure pipes **are included** in the stored [`TokenStream`].
/// 5. [`children`](Children): Either no children (ends with `;`) or a children
///    block `{ ... }` that contains more elements/values.
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
///
/// Whether the element is a slot or not is distinguished by
/// [`Child`](crate::ast::Child).
///
/// # Parsing
/// Parsing will return an [`Err`] if parsing the [`Tag`] fails (i.e. the next
/// token is not an ident; however, will abort if a component is found +
/// generics fail). If anything else fails, parsing will **abort**.
pub struct Element {
    tag: Tag,
    selectors: SelectorShorthands,
    attrs: Attrs,
    children_args: Option<TokenStream>,
    children: Option<Children>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag: Tag = input.parse()?;
        let selectors: SelectorShorthands = input.parse().unwrap_or_abort();
        let attrs: Attrs = input.parse().unwrap_or_abort();

        if input.peek(Token![;]) {
            // no children, terminated by semicolon.
            input.parse::<Token![;]>().unwrap();
            Ok(Self::new(tag, selectors, attrs, None, None))
        } else if input.is_empty() {
            // allow no ending token if its the last child
            // makes for better editing experience when writing sequentially,
            // as syntax highlighting/autocomplete doesn't work if macro
            // can't fully compile.

            let last_span = attrs.last().map_or(
                selectors.last().map_or(tag.span(), SelectorShorthand::span),
                Attr::span,
            );
            emit_error!(
                span::join(tag.span(), last_span), "unterminated element";
                help = "add a `;` to terminate the element with no children"
            );
            Ok(Self::new(tag, selectors, attrs, None, None))
        } else if input.peek(syn::token::Brace) {
            // has children in brace.
            let (children, _) = parse::parse_braced::<Children>(input).unwrap_or_abort();
            Ok(Self::new(tag, selectors, attrs, None, Some(children)))
        } else if input.peek(Token![|]) {
            // extra args for the children
            let args = parse_closure_args(input).unwrap_or_abort();
            let children = if input.peek(syn::token::Brace) {
                Some(parse::parse_braced::<Children>(input).unwrap_or_abort().0)
            } else {
                // continue trying to parse as if there are no children
                emit_error!(
                    input.span(),
                    "expected children block after closure arguments"
                );
                None
            };
            Ok(Self::new(tag, selectors, attrs, Some(args), children))
        } else {
            // add error at the unknown token
            // continue trying to parse as if there are no children
            emit_error!(input.span(), "unknown attribute");
            emit_error!(
                span::join(tag.span(), input.span()), "child elements not found";
                help = "add a `;` at the end to terminate the element"
            );
            Ok(Self::new(tag, selectors, attrs, None, None))
        }
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(xml_to_tokens(self).unwrap_or_else(|| {
            component_to_tokens::<false>(self).expect("element should be a component")
        }));
    }
}

impl Element {
    pub const fn new(
        tag: Tag,
        selectors: SelectorShorthands,
        attrs: Attrs,
        children_args: Option<TokenStream>,
        children: Option<Children>,
    ) -> Self {
        Self {
            tag,
            selectors,
            attrs,
            children_args,
            children,
        }
    }

    pub const fn tag(&self) -> &Tag { &self.tag }

    pub const fn selectors(&self) -> &SelectorShorthands { &self.selectors }

    pub const fn attrs(&self) -> &Attrs { &self.attrs }

    pub const fn children_args(&self) -> Option<&TokenStream> { self.children_args.as_ref() }

    pub const fn children(&self) -> Option<&Children> { self.children.as_ref() }
}

/// Parses closure arguments like `|binding|` or `|(index, item)|`.
///
/// Patterns are supported within the closure.
///
/// # Parsing
/// If the first pipe is not found, an [`Err`] will be returned. Otherwise,
/// tokens are parsed until a second `|` is found. Aborts if a second `|` is not
/// found.
///
/// This is ok because closure params take a
/// [*PatternNoTopAlt*](https://doc.rust-lang.org/beta/reference/expressions/closure-expr.html),
/// so no other `|` characters are allowed within a pattern that is outside of a
/// nested group.
fn parse_closure_args(input: ParseStream) -> syn::Result<TokenStream> {
    let first_pipe = input.parse::<Token![|]>()?;

    let mut tokens = TokenStream::new();
    first_pipe.to_tokens(&mut tokens);

    loop {
        // parse until second `|` is found
        if let Ok(pipe) = input.parse::<Token![|]>() {
            pipe.to_tokens(&mut tokens);
            break Ok(tokens);
        } else if let Ok(tt) = input.parse::<TokenTree>() {
            tokens.append(tt);
        } else {
            abort!(first_pipe.span, "closure arguments not closed");
        }
    }
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

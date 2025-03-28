use proc_macro2::Span;
use proc_macro_error2::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use crate::{parse::rollback_err, span};

/// The `!DOCTYPE html;` element.
///
/// This will successfully parse as soon as a `!` is found at a child position.
/// If the rest is not given, errors will be shown with hints on how to complete
/// it.
pub struct Doctype {
    bang: Token![!],
    doctype: Option<syn::Ident>,
    html: Option<syn::Ident>,
    semi: Option<Token![;]>,
}

impl Doctype {
    pub fn span(&self) -> Span {
        let last_tok = self
            .semi
            .map(|s| s.span)
            .or(self.html.as_ref().map(|h| h.span()))
            .or(self.doctype.as_ref().map(|d| d.span()))
            .unwrap_or(self.bang.span);

        span::join(self.bang.span, last_tok)
    }
}

impl Parse for Doctype {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let Some(bang) = rollback_err(input, <Token![!]>::parse) else {
            return Err(input.error("expected ! to start DOCTYPE"));
        };

        Ok(Self {
            bang,
            doctype: rollback_err(input, syn::Ident::parse),
            html: rollback_err(input, syn::Ident::parse),
            semi: rollback_err(input, <Token![;]>::parse),
        })
    }
}

impl ToTokens for Doctype {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let doctype_span = self
            .doctype
            .as_ref()
            .map(|d| d.span())
            .unwrap_or(self.bang.span);
        let html_span = self.html.as_ref().map(|h| h.span()).unwrap_or(doctype_span);

        if self
            .doctype
            .as_ref()
            .is_none_or(|d| d.to_string() != "DOCTYPE")
        {
            emit_error!(
                doctype_span,
                "expected `DOCTYPE` after `!`";
                help = "Add `!DOCTYPE html;`"
            );
        } else if self.html.as_ref().is_none_or(|h| h.to_string() != "html") {
            emit_error!(
                html_span,
                "expected `html` after `!DOCTYPE`";
                help = "Add `!DOCTYPE html;`"
            );
        } else if self.semi.is_none() {
            emit_error!(
                html_span,
                "expected `;` after `!DOCTYPE html`";
                help = "Add `!DOCTYPE html;`"
            )
        }

        let doctype_fn = quote_spanned!(doctype_span=> doctype);
        let eq = quote_spanned!(self.bang.span=> =);
        // there will never be an error on these so call site is fine
        let partial_doctype = self
            .doctype
            .clone()
            .unwrap_or(syn::Ident::new("DOCTYPE", Span::call_site()));
        let partial_html = self
            .html
            .clone()
            .unwrap_or(syn::Ident::new("html", Span::call_site()));

        // don't span "html" so they aren't string colored.
        // "html" can't have an error anyways.
        tokens.extend(quote! {
            {
                // suggest autocomplete DOCTYPE and html
                #[allow(non_snake_case)]
                let DOCTYPE = ();
                let html = ();
                let _: () #eq #partial_doctype;
                let _: () = #partial_html;
                ::leptos::tachys::html::#doctype_fn("html")
            }
        });
    }
}

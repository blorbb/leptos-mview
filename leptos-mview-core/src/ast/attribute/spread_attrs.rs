use proc_macro2::Span;
use syn::{
    braced,
    parse::{discouraged::Speculative, Parse},
    Token,
};

#[derive(Debug, Clone)]
pub struct SpreadAttr {
    ident: syn::Ident,
    span: Span,
}

impl Parse for SpreadAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // try parse spread attributes `{..attrs}`
        let fork = input.fork();
        let stream;
        let brace_token = braced!(stream in fork);
        if stream.peek(Token![..]) && stream.peek3(syn::Ident) {
            _ = stream.parse::<Token![..]>();
            let ident = stream.parse::<syn::Ident>()?;
            // if not empty, do not parse
            if stream.is_empty() {
                input.advance_to(&fork);
                return Ok(Self {
                    ident,
                    span: brace_token.span.join(),
                });
            };
        };
        Err(input.error("invalid spread attribute"))
    }
}

impl SpreadAttr {
    pub const fn as_ident(&self) -> &syn::Ident { &self.ident }

    pub const fn span(&self) -> Span { self.span }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::SpreadAttr;

    #[test]
    fn compiles() { let _: SpreadAttr = parse_quote!({ ..a }); }
}

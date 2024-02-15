use syn::parse::{discouraged::Speculative, ParseStream};

pub fn rollback_err<F, T>(input: ParseStream, parser: F) -> Option<T>
where
    F: Fn(ParseStream) -> syn::Result<T>,
{
    let fork = input.fork();
    match parser(&fork) {
        Ok(val) => {
            input.advance_to(&fork);
            Some(val)
        }
        Err(_) => None,
    }
}

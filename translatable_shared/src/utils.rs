use std::path::Path;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub fn is_ident(candidate: &str) -> bool {
    let mut chars = candidate.chars();

    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false
    }

    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[inline(always)]
pub fn option_stream<T: ToTokens>(opt: &Option<T>) -> TokenStream {
    match opt {
        Some(val) => quote! { ::std::option::Option::Some(#val) },
        None => quote! { ::std::option::Option::None }
    }
}

pub fn path_to_tokens(path: &Path) -> TokenStream {
    let path = path.to_string_lossy();
    quote! { ::std::path::Path::from(#path) }
}

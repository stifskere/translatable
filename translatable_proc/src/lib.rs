use proc_macro::TokenStream;

mod generation;
mod parsing;

#[proc_macro]
pub fn translation(tokens: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro_derive(TranslationContext, attributes(path, base_path))]
pub fn translation_context(structure: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro]
pub fn translatable_config(tokens: TokenStream) -> TokenStream {
    todo!()
}

use proc_macro2::TokenStream;
use quote::ToTokens;

pub enum ItemType<T> {
    Static(T),
    Dynamic(TokenStream)
}

impl<T: ToTokens> ItemType<T> {
    pub fn dynamic(self) -> TokenStream {
        match self {
            Self::Static(s) => s.into_token_stream(),
            Self::Dynamic(d) => d
        }
    }
}

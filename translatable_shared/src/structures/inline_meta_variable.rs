use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::{Parse, ParseStream}, parse2, Expr, Ident, Result as SynResult, Token};

pub struct InlineMetaVariable {
    key: String,
    value: TokenStream
}

impl InlineMetaVariable {
    #[inline(always)]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[inline(always)]
    pub fn value(&self) -> TokenStream {
        self.value.clone()
    }

    #[inline(always)]
    pub fn value_as<T: Parse>(&self) -> SynResult<T> {
        parse2(self.value.clone())
    }
}

impl Parse for InlineMetaVariable {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let key = input.parse::<Ident>()?;
        let value = if input.parse::<Token![=]>().is_ok() {
            input.parse::<Expr>()?.to_token_stream()
        } else {
            key.to_token_stream()
        };

        Ok(Self { key: key.to_string(), value })
    }
}

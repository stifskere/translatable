use std::str::FromStr;
use std::ops::Range;
use std::collections::HashMap;

use thiserror::Error;

#[cfg(feature = "internal")]
use ::{
    proc_macro2::TokenStream,
    quote::{quote, ToTokens, TokenStreamExt}
};

use crate::utils::is_ident;

#[derive(Error, Debug, Clone)]
pub enum TemplateParseError {
    #[error(r#"
    Found unclosed template brace at index {0}.
    If you intended to escape the brace,
    use "{{{{" instead.
    "#)]
    Unclosed(usize),
 
    #[error(r#"
    Found a template with key '{0}' which is
    an invalid identifier. Identifiers must start with
    a letter or underscore, and end with many letters,
    digits, or underscores.
    "#)]
    InvalidIdent(String)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplatedString {
    original: String,
    spans: Vec<(String, Range<usize>)>
}

#[cfg(feature = "internal")]
impl ToTokens for TemplateParseError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { ::translatable::prelude::TemplateParseError:: });

        tokens.extend(
            match self {
                Self::Unclosed(index) => quote! { Unclosed(#index) },
                Self::InvalidIdent(ident) => quote! { InvalidIdent(#ident.to_string()) },
            }
        )
    }
}

impl TemplatedString {
    #[inline(always)]
    pub fn new(original: &str, replacements: Vec<(String, Range<usize>)>) -> Self {
        Self { original: original.to_string(), spans: replacements }
    }

    pub fn replace_with(&self, replacements: &HashMap<String, String>) -> String {
        let mut result = self.original.clone();
        let mut spans = self.spans.clone();

        spans.sort_by_key(|(_, range)| range.start);

        let mut offset = 0isize;

        for (key, range) in spans {
            if let Some(value) = replacements.get(&key) {
                let start = (range.start as isize + offset) as usize;
                let end = (range.end as isize + offset) as usize;

                result.replace_range(start..end, value);

                offset += value.len() as isize - (range.end - range.start) as isize;
            }
        }

        result
    }

    #[inline(always)]
    pub fn original(&self) -> &str {
        &self.original
    }
}

impl FromStr for TemplatedString {
    type Err = TemplateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let original = s.to_string();
        let mut spans = Vec::new();

        let character_bytes = s
            .char_indices()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let mut last_bracket_idx = None;
        let mut current_tmpl_key = String::new();

        for (char_idx, c) in original.chars().enumerate() {
            match (c, last_bracket_idx) {
                ('{', Some(prev)) if prev == char_idx.saturating_sub(1) => last_bracket_idx = None,
                ('{', _) => last_bracket_idx = Some(char_idx),

                ('}', Some(open_idx)) => {
                    let key = current_tmpl_key.clone();

                    spans.push((
                        is_ident(&key)
                            .then_some(key.clone())
                            .ok_or(TemplateParseError::InvalidIdent(key))?,
                        character_bytes[open_idx]
                            ..character_bytes
                                .get(char_idx + 1)
                                .copied()
                                .unwrap_or_else(|| s.len())
                    ));

                    last_bracket_idx = None;
                    current_tmpl_key.clear();
                }

                (c, Some(_)) => current_tmpl_key.push(c),

                _ => {}
            }
        }

        if let Some(lbi) = last_bracket_idx {
            Err(TemplateParseError::Unclosed(lbi))
        } else {
            Ok(Self { original, spans })
        }
    }
}

#[cfg(feature = "internal")]
impl ToTokens for TemplatedString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let original = &self.original;

        let span_map = self
            .spans
            .iter()
            .map(|(key, range)| {
                let start = range.start;
                let end = range.end;

                quote! { (#key.to_string(), #start..#end) }
            });

        tokens.append_all(quote! {
            ::translatable::prelude::LocalizedString::new(
                #original,
                vec![#(#span_map),*]
            )
        })
    }
}

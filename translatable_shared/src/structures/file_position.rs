use std::ops::Range;
use std::fmt::{Display, Formatter, Result as FmtResult};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileLocation {
    line: usize,
    column: usize
}

impl FileLocation {
    #[inline(always)]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub const fn zero() -> Self {
        Self { line: 0, column: 0 }
    }

    #[inline(always)]
    pub const fn line(&self) -> usize {
        self.line
    }

    #[inline(always)]
    pub const fn column(&self) -> usize {
        self.column
    }

    pub fn from_byte_index(text: &str, index: usize) -> Option<Self> {
        if index >= text.len() {
            return None;
        }

        text.lines()
            .enumerate()
            .fold((0, None), |(offset, result), (line_num, line)| {
                if result.is_some() {
                    return (offset, result);
                }

                let line_len = line.len() + 1;
                if index < offset + line_len {
                    (offset, Some((line_num, index - offset)))
                } else {
                    (offset + line_len, None)
                }
            })
                .1
                .map(Into::into)
    }

    pub fn from_optional_range(text: &str, span: Option<Range<usize>>) -> Self {
        Self::from_byte_index(
            text,
            span.map_or(0, |span| span.start)
        )
            .unwrap_or_else(Self::zero)
    }
}

impl<L: Into<usize>, C: Into<usize>> From<(L, C)> for FileLocation {
    #[inline(always)]
    fn from((line, column): (L, C)) -> Self {
        Self {
            line: line.into(),
            column: column.into()
        }
    }
}

impl Into<(usize, usize)> for FileLocation {
    #[inline(always)]
    fn into(self) -> (usize, usize) {
        (self.line, self.column)
    }
}

impl Display for FileLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl ToTokens for FileLocation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let line = self.line;
        let column = self.column;

        tokens.extend(quote! {
            ::translatable::prelude::LocationInFile::new(#line, #column)
        })
    }
}

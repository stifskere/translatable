use proc_macro2::Span;
use syn::{Ident, Result as SynResult, Token};
use syn::parse::{Parse, ParseStream};

pub struct Path {
    is_root: bool,
    segments: Vec<String>,
    span: Span
}

impl Path {
    pub fn merge(self, other: Self) -> Option<Self> {
        Some(Path {
            is_root: self.is_root,
            segments: [self.segments, other.segments].concat(),
            span: self.span.join(other.span)?
        })
    }

    #[inline(always)]
    pub const fn is_root(&self) -> bool {
        self.is_root
    }

    #[inline(always)]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    #[inline(always)]
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl Parse for Path {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(Self {
            is_root: input
                .parse::<Token![::]>()
                .is_ok(),
            segments: input
                .parse_terminated(Ident::parse, Token![::])?
                .into_iter()
                .map(|segment| segment.to_string())
                .collect(),
            span: input.span()
        })
    }
}

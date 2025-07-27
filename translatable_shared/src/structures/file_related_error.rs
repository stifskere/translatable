use std::path::PathBuf;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::error::Error;

#[cfg(feature = "internal")]
use ::{
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
};

use crate::structures::file_position::FileLocation;

#[cfg(feature = "internal")]
use crate::utils::internal::{option_stream, path_to_tokens};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileRelatedError<TDesc: Sized + Display> {
    pub(crate) description: TDesc,
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) at_character: Option<FileLocation>
}

impl<TDesc: Sized + Display> FileRelatedError<TDesc> {
    #[inline(always)]
    #[doc(hidden)]
    pub const fn from_data(
        description: TDesc,
        file_path: Option<PathBuf>,
        at_character: Option<FileLocation>
    ) -> Self {
        Self {
            description,
            file_path,
            at_character
        }
    }

    #[cfg(feature = "preparsing")]
    #[inline(always)]
    pub const fn with_desc_only(description: TDesc) -> Self {
        Self::from_data(description, None, None)
    }

    #[cfg(feature = "preparsing")]
    #[inline(always)]
    pub const fn with_desc_and_path(description: TDesc, file_path: Option<PathBuf>) -> Self {
        Self::from_data(description, file_path, None)
    }

    // NOTE: binding for public API
    #[cfg(feature = "preparsing")]
    #[inline(always)]
    pub const fn complete(
        description: TDesc,
        file_path: Option<PathBuf>,
        at_character: Option<FileLocation>
    ) -> Self {
        Self::from_data(
            description,
            file_path,
            at_character
        )
    }

    #[inline(always)]
    pub fn description(&self) -> &TDesc {
        &self.description
    }

    #[inline(always)]
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    #[inline(always)]
    pub fn at_character(&self) -> &Option<FileLocation> {
        &self.at_character
    }
}

impl<TDesc: Sized + Display> Display for FileRelatedError<TDesc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{:#}",
            self.description
        )?;

        let file_path = self.file_path()
            .and_then(|path| path.to_str());
        let at_character = self.at_character
            .map(|position| position.to_string());

        match (file_path, at_character) {
            (Some(file_path), Some(at_character)) => write!(
                f, "\nAt {file_path}:{at_character}."
            )?,

            (Some(file_path), None) => write!(
                f, "\nAt {file_path}."
            )?,

            _ => {}
        };

        Ok(())
    }
}

#[cfg(feature = "internal")]
impl<TDesc: Sized + Display + ToTokens> ToTokens for FileRelatedError<TDesc> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = &self.description;
        let file_path = option_stream(&self.file_path().map(|p| path_to_tokens(p)));
        let at_character = option_stream(&self.at_character);

        tokens.extend(quote! {
            ::translatable::prelude::TranslationparseError::new(
                #description,
                ::std::path::PathBuf::from(#file_path.to_string()),
                #at_character
            )
        })
    }
}

impl<TDesc: Sized + Debug + Display> Error for FileRelatedError<TDesc> {}

impl<TDesc: Sized + Display + Clone> Clone for FileRelatedError<TDesc> {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            file_path: self.file_path.clone(),
            at_character: self.at_character
        }
    }
}

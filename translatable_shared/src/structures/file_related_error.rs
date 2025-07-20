use std::path::PathBuf;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::error::Error;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::structures::file_position::FileLocation;
use crate::utils::{option_stream, path_to_tokens};

#[expect(unused_macros)]
macro_rules! file_related_error {
    ([$desc:expr] in [$file_path:expr] in [$at_character:expr]) => {
        $crate::structures::file_related_error::FileRelatedError {
            description: $desc,
            file_path: Some($file_path),
            at_character: Some($at_character)
        }
    };

    ([$desc:expr] in [$file_path:expr]) => {
        $crate::structures::file_related_error::FileRelatedError {
            description: $desc,
            file_path: Some($file_path),
            at_character: None
        }
    };

    ([$desc:expr]) => {
        $crate::structures::file_related_error::FileRelatedError {
            description: $desc,
            file_path: None,
            at_character: None
        }
    }
}

pub(crate) use file_related_error;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileRelatedError<TDesc: Sized + Display> {
    pub(crate) description: TDesc,
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) at_character: Option<FileLocation>
}

impl<TDesc: Sized + Display> FileRelatedError<TDesc> {
    #[inline(always)]
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

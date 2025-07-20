use std::collections::HashMap;
use std::fs::read_to_string;
use std::hash::{Hash, Hasher};
use std::path::Path;

use edit_distance::edit_distance;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use thiserror::Error;
use toml_edit::{DocumentMut, Item as TomlItem, Table as TomlTable, Value as TomlValue};

use crate::structures::language::{Language, LanguageError};
use crate::structures::file_position::FileLocation;
use crate::structures::templated_string::{TemplateParseError, TemplatedString};
use crate::structures::file_related_error::{FileRelatedError};

#[derive(Error, Debug, Clone)]
pub enum TranslationTreeErrorDescription {
    #[error(r#"
    An invalid value was found in a nesting, nestings only support
    other nestings or translations. For which a translation is a map
    of languages (valid iso-639-1 strings) and raw strings and a nesting
    is a subset of structures containing other structures.
    "#)]
    InvalidValueInNesting,

    #[error(r#"
    An invalid value was found in a translation, translations are maps for
    which keys are languages (valid iso-639-1 strings) and raw strings that
    represent a version of a string in that specific language.
    "#)]
    InvalidValueInTranslation,

    #[error(r#"
    Found an empty table, can't infer wether it's a translation or an object.
    Either remove that empty table or fill it with a valid value.
    "#)]
    EmptyTable,

    #[error(r#"
    Found an invalid language in a translation object key.
    {0:#}
    "#)]
    InvalidLanguageKey(#[from] LanguageError),

    #[error(r#"
    An error occurred while opening a file
    related to this translation branch.
    {0:#}
    "#)]
    IoError(String),

    #[error(r#"
    An error occurred while parsing TOML for
    content related to this translation branch.
    "{0:#}"
    "#)]
    // error description
    TomlError(String),

    #[error(r#"
    An error occurred while parsing templates
    for a translation string in this branch.
    "#)]
    TemplateError(#[from] TemplateParseError)
}

type TranslationTreeParseError = FileRelatedError<TranslationTreeErrorDescription>;

#[derive(Error, Debug)]
pub enum TranslationNotFound {
    #[error(
        r#"
        "{attempted_segment}" does not exist in "{accomplished_segments}"{next_possibility}
        "#,
        accomplished_segments = accomplished_segments.join("::"),
        next_possibility = if let Some(next) = closest_next_possibility {
            format!(r#", perhaps you meant "{next}"?"#)
        } else {
            ".".to_string()
        }
    )]
    NotFoundInNode {
        accomplished_segments: Vec<String>,
        attempted_segment: String,
        closest_next_possibility: Option<String>,
    },

    #[error(
        r#"
        Attempted to access "{attempted_segment}" at "{accomplished_segments}",
        but failed because "{accomplished_segments}" leads to a translation object
        and not a nesting.
        "#,
        accomplished_segments = accomplished_segments.join("::")
    )]
    FoundEarlyTranslation {
        accomplished_segments: Vec<String>,
        attempted_segment: String,
    },

    #[error(
        r#"
        The path "{accomplished_segments}" is invalid or may be incomplete,
        because "{accomplished_segments}" leads to a nesting, not a translation.
        "#,
        accomplished_segments = accomplished_segments.join("::")
    )]
    PathLeadsToNesting {
        accomplished_segments: Vec<String>,
    },

    #[error("This node is not accessible because an error occurred while parsing it.\n{0:#}")]
    FoundErroredNode(#[from] TranslationTreeParseError)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Translation(HashMap<Language, TemplatedString>);

#[non_exhaustive]
#[derive(Debug)]
pub enum TranslationTree {
    #[non_exhaustive] Nesting(HashMap<String, TranslationTree>),
    #[non_exhaustive] Translation(Translation),
    #[non_exhaustive] NestingError(TranslationTreeParseError)
}

impl ToTokens for TranslationTreeErrorDescription {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { ::translatable::prelude::TreeErrorDescription:: });

        tokens.extend(
            match self {
                Self::InvalidValueInNesting
                    => quote! { InvalidValueInNesting },

                Self::InvalidValueInTranslation
                    => quote! { InvalidValueInTranslation },

                Self::EmptyTable
                    => quote! { EmptyTable },

                Self::InvalidLanguageKey(error)
                    => quote! { InvalidLanguageKey(#error) },

                Self::IoError(error)
                    => quote! { IoError(#error.to_string()) },

                Self::TomlError(error)
                    => quote! { TomlError(#error.to_string()) },

                Self::TemplateError(error)
                    => quote! { TemplateError(#error) }
            }
        );
    }
}

impl Translation {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    #[inline(always)]
    pub const fn from_data(raw: HashMap<Language, TemplatedString>) -> Self {
        Self(raw)
    }

    pub fn available_languages(&self) -> Vec<&Language> {
        self.0
            .keys()
            .collect()
    }

    #[inline(always)]
    pub fn get_language(&self, lang: &Language) -> Option<&TemplatedString> {
        self.0
            .get(&lang)
    }

    pub fn language_available(&self, lang: &Language) -> bool {
        self.0
            .contains_key(&lang)
    }

    pub(crate) fn insert(&mut self, key: Language, value: TemplatedString) {
        self.0
            .insert(key, value);
    }
}

impl ToTokens for Translation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let possibilities = self.0
            .iter()
            .map(|(key, value)| quote! { (#key, #value.to_string()) });

        tokens.extend(
            quote! {
                ::translatable::shared::Translation::new(
                    ::std::collections::HashMap::from([
                        #(#possibilities),*
                    ])
                )
            }
        )
    }
}

impl Hash for Translation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            state.write(&key.to_string().into_bytes());
            value.hash(state);
        }
    }
}

impl TranslationTree {
    #[inline(always)]
    pub fn empty_translation() -> Self {
        Self::Translation(Translation::empty())
    }

    #[inline(always)]
    pub fn empty_nesting() -> Self {
        Self::Nesting(HashMap::new())
    }

    fn collect_translations(source: Option<&Path>, raw_contents: &str, table: &TomlTable) -> Self {
        let mut result = None;

        // HACK: This may be rewritten and should use less nesting
        // i.e use error-first logic.
        for (key, value) in table {
            match value {
                TomlItem::Value(TomlValue::String(value)) => {
                    match result.get_or_insert_with(|| Self::Translation(Translation::default())) {
                        Self::Translation(translation) => {
                            let language = match key.parse::<Language>() {
                                Ok(language) => language,

                                Err(error) => {
                                    result = Some(Self::NestingError(TranslationTreeParseError {
                                        description: TranslationTreeErrorDescription::InvalidLanguageKey(error),
                                        file_path: source.map(|p| p.to_path_buf()),
                                        at_character: Some(
                                            FileLocation::from_optional_range(raw_contents, value.span())
                                        )
                                    }));

                                    continue;
                                }
                            };

                            match value.value().parse() {
                                Ok(templated_string) => translation.insert(
                                    language,
                                    templated_string
                                ),

                                Err(error) => {
                                    result = Some(Self::NestingError(TranslationTreeParseError {
                                        description: TranslationTreeErrorDescription::TemplateError(error),
                                        file_path: source.map(|p| p.to_path_buf()),
                                        at_character: Some(
                                            FileLocation::from_optional_range(raw_contents, value.span())
                                        )
                                    }));
                                }
                            }
                        }

                        Self::Nesting(_) => {
                            result = Some(Self::NestingError(TranslationTreeParseError {
                                description: TranslationTreeErrorDescription::InvalidValueInTranslation,
                                file_path: source.map(|p| p.to_path_buf()),
                                at_character: Some(
                                    FileLocation::from_optional_range(raw_contents, value.span())
                                )
                            }))
                        }

                        Self::NestingError(_) => { break; }
                    }
                }

                TomlItem::Table(value) => {
                    match result.get_or_insert_with(|| Self::Nesting(HashMap::new())) {
                        Self::Nesting(result) => {
                            result.insert(
                                key.to_string(),
                                Self::collect_translations(source, raw_contents, value)
                            );
                        }

                        Self::Translation(_) => {
                            result = Some(Self::NestingError(TranslationTreeParseError {
                                description: TranslationTreeErrorDescription::InvalidValueInNesting,
                                file_path: source.map(|p| p.to_path_buf()),
                                at_character: Some(
                                    FileLocation::from_optional_range(raw_contents, value.span())
                                )
                            }))
                        },

                        Self::NestingError(_) => { break; }
                    }
                }

                other => {
                    result = Some(Self::NestingError(TranslationTreeParseError {
                        description: TranslationTreeErrorDescription::InvalidValueInNesting,
                        file_path: source.map(|p| p.to_path_buf()),
                        at_character: Some(
                            FileLocation::from_optional_range(raw_contents, other.span())
                        )
                    }))
                }
            }
        }

        result.unwrap_or_else(|| {
            Self::NestingError(TranslationTreeParseError {
                description: TranslationTreeErrorDescription::EmptyTable,
                file_path: source.map(|p| p.to_path_buf()),
                at_character: Some(
                    FileLocation::from_optional_range(raw_contents, table.span())
                )
            })
        })
    }

    pub fn from_raw(source: Option<&Path>, raw_contents: &str) -> Self {
        let toml_contents = match raw_contents.parse::<DocumentMut>() {
            Ok(contents) => contents,

            Err(error) => {
                return Self::NestingError(TranslationTreeParseError {
                    description: TranslationTreeErrorDescription::TomlError(error.to_string()),
                    file_path: source.map(|s| s.to_path_buf()),
                    at_character: Some(
                        FileLocation::from_optional_range(raw_contents, error.span())
                    )
                });
            }
        };

        let table = match toml_contents.as_item() {
            TomlItem::Table(table) => table,

            other => {
                return Self::NestingError(TranslationTreeParseError {
                    description: TranslationTreeErrorDescription::InvalidValueInNesting,
                    file_path: source.map(|s| s.to_path_buf()),
                    at_character: Some(
                        FileLocation::from_optional_range(raw_contents, other.span())
                    )
                });
            }
        };

        Self::collect_translations(source, raw_contents, table)
    }

    pub fn fallback_available(&self, language: Language) -> bool {
        match self {
            Self::Nesting(nesting) => {
                for (_, tree) in nesting {
                    if tree.fallback_available(language) {
                        return true;
                    }
                }
            }

            Self::Translation(translation) => {
                if translation.language_available(&language) {
                    return true;
                }
            },

            // errors don't short circuit the lookup.
            Self::NestingError(_) => {}
        }

        false
    }

    fn closest_next_nesting(&self, key: &str) -> Option<String> {
        match self {
            Self::Nesting(nesting) => {
                nesting
                    .keys()
                    .min_by_key(|candidate| edit_distance(key, candidate))
                    .cloned()
            }

            Self::Translation(_) | Self::NestingError(_) => None,
        }
    }

    pub fn find_path<I: Iterator<Item = String>>(&self, mut segments: I) -> Result<&Translation, TranslationNotFound> {
        let mut accomplished = Vec::new();
        let mut tree_node = self;

        while let Some(segment) = segments.next() {
            match self {
                Self::Nesting(nesting) => {
                    match nesting.get(&segment) {
                        Some(next) => {
                            tree_node = next;
                            accomplished.push(segment.clone());
                        }

                        None => {
                            return Err(TranslationNotFound::NotFoundInNode {
                                accomplished_segments: accomplished,
                                closest_next_possibility: tree_node.closest_next_nesting(&segment),
                                attempted_segment: segment
                            });
                        }
                    }
                },

                Self::Translation(translation) => {
                    if segments.next().is_some() {
                        return Err(TranslationNotFound::FoundEarlyTranslation {
                            accomplished_segments: accomplished,
                            attempted_segment: segment
                        });
                    }

                    return Ok(translation);
                },

                Self::NestingError(error) => {
                    return Err(error.clone().into());
                }
            }
        }

        Err(TranslationNotFound::PathLeadsToNesting {
            accomplished_segments: accomplished
        })
    }
}

impl From<&Path> for TranslationTree {
    fn from(path: &Path) -> Self {
        match read_to_string(path) {
            Ok(raw_contents) => Self::from_raw(Some(path), &raw_contents),

            Err(error) => {
                Self::NestingError(TranslationTreeParseError {
                    description: TranslationTreeErrorDescription::IoError(error.to_string()),
                    file_path: Some(path.to_path_buf()),
                    at_character: None
                })
            }
        }
    }
}

impl From<&str> for TranslationTree {
    fn from(raw_contents: &str) -> Self {
        Self::from_raw(None, raw_contents)
    }
}

impl ToTokens for TranslationTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Nesting(nesting) => {
                let nesting_tokens = nesting
                    .into_iter()
                    .map(|(key, value)| quote! { (#key.to_string(), #value) });

                tokens.extend(quote! {
                    ::translatable::prelude::TranslationTree::Nesting(
                        ::std::collections::HashMap::from([
                            #(#nesting_tokens),*
                        ])
                    )
                })
            }

            Self::Translation(translation) => tokens.extend(quote! {
                ::translatable::prelude::TranslationTree::Translation(#translation)
            }),

            Self::NestingError(error) => tokens.extend(quote! {
                ::translatable::prelude::TranslationTree::NestingError(#error)
            }),
        }
    }
}

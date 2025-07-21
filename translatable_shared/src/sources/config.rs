#![cfg(feature = "internal")]

use std::fs::read_to_string;
use std::env::var;
use std::path::Path;
use std::sync::OnceLock;
use std::io::Error as IoError;

use thiserror::Error;
use toml_edit::{TomlError, DocumentMut};
use dyn_path::{dyn_access, dyn_path};

use crate::structures::file_related_error::FileRelatedError;
use crate::structures::file_position::FileLocation;
use crate::structures::language::{Language, LanguageError};

#[derive(Error, Debug)]
pub enum ConfigErrorDescription {
    #[error("Couldn't open configuration file.\n{0:#}")]
    SystemIo(IoError),

    #[error("Error while parsing configuration TOML file.\n{0:#}")]
    ParseToml(TomlError),

    #[error(
        r#"
        Couldn't parse configuration, expected "{item_path}"
        to be of a type that could be parsed into a '{expected_type}',
        but found a value which can't be parsed to that type.
        {error_display}
        "#,
    )]
    ParseValue {
        item_path: String,
        expected_type: String,
        error_display: String
    }
}

pub(super) type ConfigError = FileRelatedError<ConfigErrorDescription>;

pub struct Config {
    // sources section
    sources_path: String, // where to get the translations from.

    // fallback section
    fallback_language: Option<Language>, // if the language does not exist, use this.
    fallback_translation: Option<String> // if unavailable language or translation use this.
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let file_path = Path::new(
            &var("TRANSLATABLE_CONFIG_PATH")
                .unwrap_or_else(|_| "./translatable.toml".into())
        )
            .to_path_buf();

        let file_path = file_path
            .canonicalize()
            .map_err(|error| ConfigError {
                description: ConfigErrorDescription::SystemIo(error),
                file_path: Some(file_path),
                at_character: None
            })?;

        let file_contents = read_to_string(&file_path)
            .map_err(|error| ConfigError {
                description: ConfigErrorDescription::SystemIo(error),
                file_path: Some(file_path.to_path_buf()),
                at_character: None
            })?;

        let table = file_contents
            .parse::<DocumentMut>()
            .map_err(|error| ConfigError {
                description: ConfigErrorDescription::ParseToml(error.clone()),
                file_path: Some(file_path.to_path_buf()),
                at_character: Some(
                    FileLocation::from_optional_range(&file_contents, error.span())
                )
            })?;

        Ok(Self {
            sources_path: dyn_access!(table.sources.path)
                .and_then(|path| path.as_str())
                .unwrap_or("./translations")
                .to_string(),

            fallback_language: (|| {
                let (raw_str, raw_span) = dyn_access!(table.fallbacks.language)
                    .and_then(|raw| Some((raw.as_str()?, raw.span())))?;

                Some(raw_str
                    .parse()
                    .map_err(|error: LanguageError| ConfigError {
                        description: ConfigErrorDescription::ParseValue {
                            item_path: dyn_path!(fallbacks.language),
                            expected_type: "::translatable::prelude::Language".into(),
                            error_display: error.to_string()
                        },
                        file_path: Some(file_path.to_path_buf()),
                        at_character: Some(FileLocation::from_optional_range(&file_contents, raw_span))
                    }))
            })()
                .transpose()?,

            fallback_translation: dyn_access!(table.fallbacks.translation)
                .and_then(|raw| raw.as_str())
                .map(ToString::to_string)
        })
    }

    pub fn load_cached() -> Result<&'static Self, ConfigError> {
        static CACHED: OnceLock<Config> = OnceLock::new();

        if let Some(cached) = CACHED.get() {
            return Ok(cached);
        }

        let loaded = Self::load()?;

        Ok(CACHED.get_or_init(|| loaded))
    }

    pub fn sources_path(&self) -> &str {
        &self.sources_path
    }

    pub const fn fallback_language(&self) -> Option<Language> {
        self.fallback_language
    }

    pub const fn fallback_translation(&self) -> Option<&String> {
        self.fallback_translation.as_ref()
    }
}

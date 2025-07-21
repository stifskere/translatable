use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::collections::HashMap;

#[cfg(feature = "internal")]
use std::sync::OnceLock;

use crate::structures::translation_tree::TranslationTree;

#[cfg(feature = "internal")]
use crate::sources::config::{Config, ConfigError};

enum TranslationTreeSource {
    File(PathBuf),
    Raw(String)
}

pub struct TranslationTreeBuilder {
    sources: HashMap<Vec<String>, TranslationTreeSource>
}

impl TranslationTreeBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            sources: HashMap::new()
        }
    }

    pub fn with_file_source(&mut self, path: Vec<String>, file: PathBuf) -> &mut Self {
        self
            .sources
            .insert(path, TranslationTreeSource::File(file));

        self
    }

    pub fn with_raw_source(&mut self, path: Vec<String>, contents: String) -> &mut Self {
        self
            .sources
            .insert(path, TranslationTreeSource::Raw(contents));

        self
    }

    pub fn build(self) -> TranslationTree {
        let mut root = HashMap::<String, TranslationTree>::new();

        for (path, source) in self.sources {
            let mut cursor = &mut root;
            let mut segments = path.iter().peekable();
            let tree = match source {
                TranslationTreeSource::File(path) => TranslationTree::from(path.as_path()),
                TranslationTreeSource::Raw(raw) => TranslationTree::from(raw.as_str())
            };

            while let Some(segment) = segments.next() {
                if segments.peek().is_some() {
                    cursor = match cursor.entry(segment.clone()) {
                        Entry::Occupied(entry) => {
                            match entry.into_mut() {
                                TranslationTree::Nesting(map) => map,
                                _ => unreachable!("Non-nesting node found when guaranted.")
                            }
                        }

                        Entry::Vacant(entry) => {
                            match entry.insert(TranslationTree::empty_nesting()) {
                                TranslationTree::Nesting(map) => map,
                                _ => unreachable!("Non-nesting node found when guaranted.")
                            }
                        }
                    }
                } else {
                    cursor.insert(segment.clone(), tree);
                    break;
                }
            }
        }

        TranslationTree::Nesting(root)
    }
}

#[cfg(feature = "internal")]
pub fn tree_from_config() -> Result<TranslationTree, ConfigError> {
    static CACHED: OnceLock<TranslationTree> = OnceLock::new();
    let config = Config::load_cached()?;

    

    unimplemented!()
}

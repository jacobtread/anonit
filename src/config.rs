use crate::{
    ctx::ContextData,
    data::{key::PathKey, value::DataValueItem},
    fake::{FakeDataProducer, FakeDataProducerFactory, prompt_fake_data_type},
};
use eyre::Context;
use inquire::prompt_confirmation;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
    sync::Arc,
};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Mapping from keys to producers
    pub mapping: HashMap<Arc<PathKey>, Box<dyn FakeDataProducer>>,
    /// Keys that will produce outputs
    pub output: HashSet<Arc<PathKey>>,
    /// Default producer for unknown keys
    pub default: Option<Box<dyn FakeDataProducer>>,

    /// Whether to maintain null values
    pub maintain_null: bool,

    /// Whether to allow mappings of redacted fields to be built
    /// and used within the current input set
    ///
    /// Helps use cases where in multiple parts of a file you have
    /// an ID that refers to a user and want all instances of the
    /// ID to be consistent
    pub internal_mapping: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mapping: Default::default(),
            output: Default::default(),
            default: Default::default(),
            maintain_null: true,
            internal_mapping: true,
        }
    }
}

impl Config {
    /// Read a config from the provided file path
    pub fn try_from_file(path: impl AsRef<Path>) -> eyre::Result<Config> {
        let file = File::open(path).context("failed to open input file")?;
        let value: Config = serde_json::from_reader(file).context("failed to read / parse file")?;
        Ok(value)
    }

    /// Prompt building a configuration from the provided structure
    pub fn prompt_from_structure(
        registry: &[Box<dyn FakeDataProducerFactory>],
        structure: &[DataValueItem],
    ) -> eyre::Result<Config> {
        let mut mapping = HashMap::new();
        let mut output = HashSet::new();
        let mut ctx = ContextData::default();

        for item in structure {
            loop {
                let producer = match prompt_fake_data_type(registry, item, &mut ctx)? {
                    Some(value) => value,
                    None => continue,
                };

                // For keys that support outputting a mapping prompt the user if they want to
                if producer.is_allowed_output() {
                    let key = item.key.to_string();
                    if prompt_confirmation(format!(
                        "Do you want to create an output mapping for {key}?"
                    ))? {
                        output.insert(item.key.clone());
                    }
                }

                mapping.insert(item.key.clone(), producer);
                break;
            }
        }

        let maintain_null = prompt_confirmation("Would you like to leave null values as is?")?;
        let internal_mapping = prompt_confirmation("Would you like to allow internal mapping?")?;

        Ok(Config {
            mapping,
            output,
            default: None,
            maintain_null,
            internal_mapping,
        })
    }
}

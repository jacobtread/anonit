use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
    sync::Arc,
};

use eyre::Context;
use inquire::prompt_confirmation;
use serde::{Deserialize, Serialize};

use crate::{
    data::{key::PathKey, value::DataValueItem},
    fake::{FakeDataProducer, FakeDataProducerFactory, prompt_fake_data_type},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub mapping: HashMap<Arc<PathKey>, Box<dyn FakeDataProducer>>,
    pub output: HashSet<Arc<PathKey>>,
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

        for item in structure {
            loop {
                let producer = match prompt_fake_data_type(registry, item)? {
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

        Ok(Config { mapping, output })
    }
}

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerData, FakeDataProducerFactory},
    prompt_utils::{prompt_file_path, prompt_range},
};
use itertools::Itertools;
use rand::{RngExt, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::read_to_string, path::PathBuf};

pub struct WordlistFakeDataFactory;

impl FakeDataProducerFactory for WordlistFakeDataFactory {
    fn name(&self) -> String {
        "Wordlist (Custom text file)".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let file_path = match prompt_file_path("Enter path to the wordlist file")? {
            Some(value) => value,
            None => return Ok(None),
        };

        if !file_path.is_file() {
            return Err(eyre::eyre!("file does not exist"));
        }

        let amount = match prompt_range(
            "Minimum number of words",
            "Maximum number of words",
            usize::MIN,
            usize::MAX,
        )? {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(WordlistFakeData { file_path, amount })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordlistFakeData {
    file_path: PathBuf,
    amount: std::ops::Range<usize>,
}

// Maintain a in-memory cache of a wordlist file to prevent loading the file
// for every single fake data generation that requires it
#[derive(Default)]
struct WordlistCache {
    cache: HashMap<PathBuf, Vec<String>>,
}

impl WordlistCache {
    fn generate(
        &mut self,
        rng: &mut impl rand::Rng,
        path: PathBuf,
        amount: usize,
    ) -> eyre::Result<String> {
        if let Some(value) = self.cache.get(&path) {
            return Ok(value.sample(rng, amount).join(" "));
        }

        let file = File::open(&path)?;
        let content = read_to_string(file)?;
        let words: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|value| value.to_owned())
            .collect();

        let value = words.sample(rng, amount).join(" ");
        self.cache.insert(path, words);
        Ok(value)
    }
}

#[typetag::serde(name = "wordlist")]
impl FakeDataProducer for WordlistFakeData {
    fn produce_fake(
        &self,
        _original_value: DataValueRef<'_>,
        data: &mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        let wordlist = data.ctx.get_or_default::<WordlistCache>();

        let amount = if self.amount.is_empty() {
            self.amount.start
        } else {
            data.rng.random_range(self.amount.clone())
        };
        let value = wordlist.generate(&mut data.rng, self.file_path.clone(), amount)?;
        Ok(DataValue::String(value))
    }
}

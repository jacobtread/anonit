use std::{cell::RefCell, collections::HashMap, fs::File, io::read_to_string, path::PathBuf};

use itertools::Itertools;
use rand::{random_range, seq::IndexedRandom};
use serde::{Deserialize, Serialize};

use crate::{
    fake::{FakeDataProducer, FakeDataProducerFactory},
    prompt_utils::{prompt_file_path, prompt_range},
};

pub struct WordlistFakeDataFactory;

impl FakeDataProducerFactory for WordlistFakeDataFactory {
    fn name(&self) -> String {
        "Wordlist (Custom text file)".to_owned()
    }

    fn prompt(
        &self,
        _item: &crate::json::JsonPathItem,
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
thread_local! {
    static WORDLIST_CACHE: RefCell<HashMap<PathBuf, Vec<String>>> = RefCell::new(HashMap::new());
}

fn get_wordlist_or_cache(path: PathBuf, amount: usize) -> eyre::Result<String> {
    let mut rng = rand::rng();

    if let Some(value) = WORDLIST_CACHE.with_borrow(|map| {
        if let Some(value) = map.get(&path) {
            return Some(value.sample(&mut rng, amount).join(" "));
        }

        None
    }) {
        return Ok(value);
    }

    let file = File::open(&path)?;
    let content = read_to_string(file)?;
    let words: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|value| value.to_owned())
        .collect();

    let value = words.sample(&mut rng, amount).join(" ");

    WORDLIST_CACHE.with_borrow_mut(|map| {
        map.insert(path, words);
        Ok(value)
    })
}

#[typetag::serde(name = "wordlist")]
impl FakeDataProducer for WordlistFakeData {
    fn produce_fake(&self, _original_value: &serde_json::Value) -> eyre::Result<serde_json::Value> {
        let amount = if self.amount.is_empty() {
            self.amount.start
        } else {
            random_range(self.amount.clone())
        };
        let value = get_wordlist_or_cache(self.file_path.clone(), amount)?;
        Ok(serde_json::Value::String(value))
    }
}

use std::fmt::Display;

use inquire::Select;
use serde::{Deserialize, Serialize};

use crate::{
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{
        email::EmailFakeDataProducerFactory, lorem::LoremIpsumFakeDataFactory,
        name::NameProducerFactory, number::NumberProducerFactory,
        number_string::NumberStringProducerFactory, uuid::UuidFakeDataFactory,
        wordlist::WordlistFakeDataFactory,
    },
};

mod email;
mod lorem;
mod name;
mod number;
mod number_string;
mod uuid;
mod wordlist;

pub trait FakeDataProducerFactory {
    /// Getter for the name of the producer
    fn name(&self) -> String;

    /// Check for whether the producer is allowed to be used with
    /// the provided item
    fn is_allowed_for(&self, _item: &DataValueItem) -> bool {
        true
    }

    /// Prompt the user for any available options and produce the fake data
    /// returning [None] considers the prompting to be cancelled allowing the
    /// user to select another producer
    fn prompt(&self, item: &DataValueItem) -> eyre::Result<Option<Box<dyn FakeDataProducer>>>;
}

#[typetag::serde(tag = "type")]
pub trait FakeDataProducer {
    fn produce_fake(&self, original_value: DataValueRef<'_>) -> eyre::Result<DataValue>;

    /// Check whether the type can be used in output mappings
    fn is_allowed_output(&self) -> bool {
        false
    }
}

struct IgnoreProducerFactory;

impl FakeDataProducerFactory for IgnoreProducerFactory {
    fn name(&self) -> String {
        "Ignore".to_owned()
    }

    fn prompt(&self, _item: &DataValueItem) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
        Ok(Some(Box::new(IgnoreProducer)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct IgnoreProducer;

#[typetag::serde(name = "ignore")]
impl FakeDataProducer for IgnoreProducer {
    fn produce_fake(&self, original_value: DataValueRef<'_>) -> eyre::Result<DataValue> {
        Ok(original_value.into())
    }
}

pub fn fake_data_registry() -> Vec<Box<dyn FakeDataProducerFactory>> {
    vec![
        Box::new(IgnoreProducerFactory),
        Box::new(LoremIpsumFakeDataFactory),
        Box::new(UuidFakeDataFactory),
        Box::new(WordlistFakeDataFactory),
        Box::new(EmailFakeDataProducerFactory),
        Box::new(NumberProducerFactory),
        Box::new(NameProducerFactory),
        Box::new(NumberStringProducerFactory),
    ]
}

struct PromptFactoryOption<'a> {
    factory: &'a dyn FakeDataProducerFactory,
}

impl<'a> Display for PromptFactoryOption<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.factory.name())
    }
}

pub fn prompt_fake_data_type<'a>(
    registry: &'a [Box<dyn FakeDataProducerFactory>],
    item: &DataValueItem,
) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
    let items: Vec<PromptFactoryOption<'a>> = registry
        .iter()
        .filter(|factory| factory.is_allowed_for(item))
        .map(|factory| PromptFactoryOption {
            factory: factory.as_ref(),
        })
        .collect();

    let key = item.key.to_string();
    let message = format!("What type should \"{key}\" be?");
    let answer = Select::new(&message, items).prompt()?;
    answer.factory.prompt(item)
}

use std::fmt::Display;

use inquire::Select;

use crate::{
    fake::{lorem::LoremIpsumFakeDataFactory, uuid::UuidFakeDataFactory},
    json::JsonPathItem,
};

mod lorem;
mod uuid;

pub trait FakeDataProducerFactory {
    /// Getter for the name of the producer
    fn name(&self) -> String;

    /// Check for whether the producer is allowed to be used with
    /// the provided item
    fn is_allowed_for(&self, _item: &JsonPathItem) -> bool {
        true
    }

    /// Prompt the user for any available options and produce the fake data
    /// returning [None] considers the prompting to be cancelled allowing the
    /// user to select another producer
    fn prompt(&self, item: &JsonPathItem) -> eyre::Result<Option<Box<dyn FakeDataProducer>>>;
}

pub trait FakeDataProducer {
    fn produce_fake(&self, original_value: &serde_json::Value) -> serde_json::Value;

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

    fn prompt(&self, _item: &JsonPathItem) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
        Ok(Some(Box::new(IgnoreProducer)))
    }
}

struct IgnoreProducer;

impl FakeDataProducer for IgnoreProducer {
    fn produce_fake(&self, original_value: &serde_json::Value) -> serde_json::Value {
        original_value.clone()
    }
}

pub fn fake_data_registry() -> Vec<Box<dyn FakeDataProducerFactory>> {
    vec![
        Box::new(IgnoreProducerFactory),
        Box::new(LoremIpsumFakeDataFactory),
        Box::new(UuidFakeDataFactory),
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
    item: &JsonPathItem,
) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
    let items: Vec<PromptFactoryOption<'a>> = registry
        .iter()
        .filter(|factory| factory.is_allowed_for(item))
        .map(|factory| PromptFactoryOption {
            factory: factory.as_ref(),
        })
        .collect();

    let key = item.path_key.to_string();
    let message = format!("What type should \"{key}\" be?");
    let answer = Select::new(&message, items).prompt()?;
    answer.factory.prompt(item)
}

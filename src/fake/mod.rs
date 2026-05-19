use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
};
use inquire::Select;
use mockall::automock;
use serde::Serialize;
use std::{fmt::Display, rc::Rc};

pub mod email;
pub mod ignore;
pub mod lorem;
pub mod name;
pub mod number;
pub mod number_string;
pub mod uuid;
pub mod wordlist;

/// Factory for producing [FakeDataProducer] instances by prompting the
/// user for the fields required to create the producer
#[automock]
pub trait FakeDataProducerFactory {
    /// Getter for the name of the producer to show in the list
    /// of producer
    fn name(&self) -> String;

    /// Check if the attached [FakeDataProducer] this factory produces
    /// can support the provided `item` type
    #[allow(unused_variables)]
    fn is_allowed_for<'i, 'd>(&self, item: &'i DataValueItem<'d>) -> bool {
        true
    }

    /// Prompt the user for any available options and produce the fake data
    /// returning [None] considers the prompting to be cancelled allowing the
    /// user to select another producer
    fn prompt<'i, 'd>(
        &self,
        item: &'i DataValueItem<'d>,
        _ctx: &'i mut ContextData,
    ) -> eyre::Result<Option<Box<dyn FakeDataProducer>>>;
}

pub struct FakeDataProducerData {
    /// Random number generator
    pub rng: rand::rngs::StdRng,
    /// Legacy rand 0.8.0 for random bigdecimal library support
    pub rng_08: rand08::rngs::StdRng,
    /// Context access for storing and accessing context data
    pub ctx: ContextData,
}

/// Producer of fake values
#[typetag::serde(tag = "type")]
#[automock]
pub trait FakeDataProducer {
    /// Produce a fake data value based on the provided `original_value`
    /// from the original source
    fn produce_fake<'i, 'd>(
        &self,
        original_value: DataValueRef<'i>,
        data: &'d mut FakeDataProducerData,
    ) -> eyre::Result<DataValue>;

    /// Check whether the type can be used in output mappings
    fn is_allowed_output(&self) -> bool {
        false
    }
}

/// FakeDataProducer is supported on shared versions of the type
impl<F: FakeDataProducer + Serialize> FakeDataProducer for Rc<F> {
    fn produce_fake<'i, 'd>(
        &self,
        original_value: DataValueRef<'i>,
        data: &'d mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        F::produce_fake(self, original_value, data)
    }

    #[doc(hidden)]
    fn typetag_name(&self) -> &'static str {
        F::typetag_name(self)
    }

    #[doc(hidden)]
    fn typetag_deserialize(&self) {
        F::typetag_deserialize(self);
    }
}

/// MockFakeDataProducer from mockall doesn't need a real serialize implementation
/// and wouldn't support one anyway so it just serializes to [IgnoreProducer]
impl Serialize for MockFakeDataProducer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ignore::IgnoreFakeData::serialize(&ignore::IgnoreFakeData, serializer)
    }
}

pub fn fake_data_registry() -> Vec<Box<dyn FakeDataProducerFactory>> {
    vec![
        Box::new(ignore::IgnoreFakeDataFactory),
        Box::new(lorem::LoremIpsumFakeDataFactory),
        Box::new(uuid::UuidFakeDataFactory),
        Box::new(wordlist::WordlistFakeDataFactory),
        Box::new(email::EmailFakeDateFactory),
        Box::new(number::NumberFakeDataFactory),
        Box::new(name::NameFakeDataFactory),
        Box::new(number_string::NumberStringFakeDataFactory),
        // TODO: Future formats
        // - Address (Street, city/state/postcode)
        // - GPS coordinates
        // - Phone numbers
        // - Dates
        // - Currency (Various currency formats like accounting format)
        // - URL
        // - IP Address
        // - Hashes
        // - Masking (Partially mask values)
        // - Format preserving (Preserve value format while randomizing the values)
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
    ctx: &mut ContextData,
) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
    let items: Vec<PromptFactoryOption<'a>> = registry
        .iter()
        .filter(|factory| factory.is_allowed_for(item))
        .map(|factory| PromptFactoryOption {
            factory: factory.as_ref(),
        })
        .collect();

    // TODO: Guessing most likely type based on the existing item

    let key = item.key.to_string();
    let message = format!("What type should \"{key}\" be?");
    let answer = Select::new(&message, items).prompt()?;
    answer.factory.prompt(item, ctx)
}

use std::fmt::Display;

use fake::Fake;
use inquire::Select;
use serde::{Deserialize, Serialize};

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerData, FakeDataProducerFactory},
    prompt_utils::prompt_range,
};

pub struct LoremIpsumFakeDataFactory;

impl FakeDataProducerFactory for LoremIpsumFakeDataFactory {
    fn name(&self) -> String {
        "Lorem Ipsum".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let unit_options = vec![
            LoremIpsumUnit::Words,
            LoremIpsumUnit::Sentences,
            LoremIpsumUnit::Paragraphs,
        ];

        let unit = match Select::new("What's unit of lorem ipsum would you like?", unit_options)
            .prompt_skippable()?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let range = match prompt_range(
            "Enter minimum unit amount",
            "Enter maximum unit amount",
            usize::MIN,
            usize::MAX,
        )? {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(LoremIpsumFakeData { unit, range })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoremIpsumFakeData {
    /// Unit to generate
    unit: LoremIpsumUnit,
    /// Range for the number of units to generate
    range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum LoremIpsumUnit {
    Words,
    Sentences,
    Paragraphs,
}

impl Display for LoremIpsumUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LoremIpsumUnit::Words => "Words",
            LoremIpsumUnit::Sentences => "Sentences",
            LoremIpsumUnit::Paragraphs => "Paragraphs",
        })
    }
}

#[typetag::serde(name = "lorem")]
impl FakeDataProducer for LoremIpsumFakeData {
    fn produce_fake(
        &self,
        _original_value: DataValueRef<'_>,
        data: &mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        Ok(match self.unit {
            LoremIpsumUnit::Words => {
                let words = fake::faker::lorem::en::Words(self.range.clone());
                let fake: Vec<String> = words.fake_with_rng(&mut data.rng);
                DataValue::String(fake.join(" "))
            }
            LoremIpsumUnit::Sentences => {
                let words = fake::faker::lorem::en::Sentence(self.range.clone());
                let fake = words.fake_with_rng(&mut data.rng);
                DataValue::String(fake)
            }
            LoremIpsumUnit::Paragraphs => {
                let words = fake::faker::lorem::en::Paragraph(self.range.clone());
                let fake = words.fake_with_rng(&mut data.rng);
                DataValue::String(fake)
            }
        })
    }
}

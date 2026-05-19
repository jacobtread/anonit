use inquire::{Text, prompt_confirmation};
use serde::{Deserialize, Serialize};

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerFactory, number::NumberRange},
};

pub struct NumberStringFakeDataFactory;

impl FakeDataProducerFactory for NumberStringFakeDataFactory {
    fn name(&self) -> String {
        "Number (String)".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let range = match NumberRange::prompt()? {
            Some(value) => value,
            None => return Ok(None),
        };

        let prefix = if prompt_confirmation("Do you want to include a prefix?")? {
            let prefix = match Text::new("Enter the prefix value").prompt_skippable()? {
                Some(value) => value,
                None => return Ok(None),
            };

            Some(prefix)
        } else {
            None
        };

        let suffix = if prompt_confirmation("Do you want to include a suffix?")? {
            let suffix = match Text::new("Enter the suffix value").prompt_skippable()? {
                Some(value) => value,
                None => return Ok(None),
            };

            Some(suffix)
        } else {
            None
        };

        Ok(Some(Box::new(NumberStringFakeData {
            range,
            prefix,
            suffix,
        })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberStringFakeData {
    range: NumberRange,
    prefix: Option<String>,
    suffix: Option<String>,
}

#[typetag::serde(name = "number_string")]
impl FakeDataProducer for NumberStringFakeData {
    fn produce_fake(
        &self,
        _original_value: DataValueRef<'_>,
        _ctx: &mut ContextData,
    ) -> eyre::Result<DataValue> {
        let value = self.range.fake()?;
        let mut string_value: String = value.into();

        if let Some(prefix) = self.prefix.as_ref() {
            string_value = format!("{prefix}{string_value}");
        }

        if let Some(suffix) = self.suffix.as_ref() {
            string_value.push_str(suffix);
        }

        Ok(DataValue::String(string_value))
    }
}

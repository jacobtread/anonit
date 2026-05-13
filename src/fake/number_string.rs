use serde::{Deserialize, Serialize};

use crate::{
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerFactory, number::NumberRange},
};

pub struct NumberStringProducerFactory;

impl FakeDataProducerFactory for NumberStringProducerFactory {
    fn name(&self) -> String {
        "Number (String)".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let range = match NumberRange::prompt()? {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(NumberStringProducer { range })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberStringProducer {
    range: NumberRange,
}

#[typetag::serde(name = "number")]
impl FakeDataProducer for NumberStringProducer {
    fn produce_fake(&self, _original_value: DataValueRef<'_>) -> eyre::Result<DataValue> {
        let value = self.range.fake()?;
        Ok(DataValue::String(value.into()))
    }
}

use fake::{Fake, faker::internet::en::SafeEmail};
use serde::{Deserialize, Serialize};

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerData, FakeDataProducerFactory},
};

pub struct EmailFakeDateFactory;

impl FakeDataProducerFactory for EmailFakeDateFactory {
    fn name(&self) -> String {
        "Email".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &crate::data::value::DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        Ok(Some(Box::new(EmailFakeData)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailFakeData;

#[typetag::serde(name = "email")]
impl FakeDataProducer for EmailFakeData {
    fn produce_fake(
        &self,
        _original_value: DataValueRef<'_>,
        data: &mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        let value = SafeEmail().fake_with_rng(&mut data.rng);
        Ok(DataValue::String(value))
    }
}

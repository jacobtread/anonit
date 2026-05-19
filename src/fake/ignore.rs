use serde::{Deserialize, Serialize};

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerData, FakeDataProducerFactory},
};

pub struct IgnoreFakeDataFactory;

impl FakeDataProducerFactory for IgnoreFakeDataFactory {
    fn name(&self) -> String {
        "Ignore".to_owned()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
        Ok(Some(Box::new(IgnoreFakeData)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgnoreFakeData;

#[typetag::serde(name = "ignore")]
impl FakeDataProducer for IgnoreFakeData {
    fn produce_fake(
        &self,
        original_value: DataValueRef<'_>,
        _data: &mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        Ok(original_value.into())
    }
}

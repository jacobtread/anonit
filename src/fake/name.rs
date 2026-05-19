use fake::{
    Fake,
    faker::{
        internet::en::Username,
        name::en::{FirstName, LastName, Name, NameWithTitle, Suffix, Title},
    },
};
use inquire::Select;
use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

use crate::{
    ctx::ContextData,
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::{FakeDataProducerData, FakeDataProducerFactory},
};

use super::FakeDataProducer;

pub struct NameFakeDataFactory;

impl FakeDataProducerFactory for NameFakeDataFactory {
    fn name(&self) -> String {
        "Name".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_string_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
        _ctx: &mut ContextData,
    ) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
        let type_options = NameStyle::VARIANTS.to_vec();
        let ty = match Select::new("What type of name would you like?", type_options)
            .prompt_skippable()?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(NameFakeData { ty })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameFakeData {
    #[serde(rename = "style")]
    ty: NameStyle,
}

#[derive(Debug, Display, VariantArray, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameStyle {
    Username,
    FirstName,
    LastName,
    Name,
    NameWithTitle,
    Suffix,
    Title,
}

#[typetag::serde(name = "name")]
impl FakeDataProducer for NameFakeData {
    fn produce_fake(
        &self,
        _original_value: DataValueRef<'_>,
        data: &mut FakeDataProducerData,
    ) -> eyre::Result<DataValue> {
        let value = match &self.ty {
            NameStyle::Username => Username().fake_with_rng(&mut data.rng),
            NameStyle::FirstName => FirstName().fake_with_rng(&mut data.rng),
            NameStyle::LastName => LastName().fake_with_rng(&mut data.rng),
            NameStyle::Name => Name().fake_with_rng(&mut data.rng),
            NameStyle::NameWithTitle => NameWithTitle().fake_with_rng(&mut data.rng),
            NameStyle::Suffix => Suffix().fake_with_rng(&mut data.rng),
            NameStyle::Title => Title().fake_with_rng(&mut data.rng),
        };

        Ok(DataValue::String(value))
    }
}

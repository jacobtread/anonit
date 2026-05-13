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
    data::value::{DataValue, DataValueItem, DataValueRef},
    fake::FakeDataProducerFactory,
};

use super::FakeDataProducer;

pub struct NameProducerFactory;

impl FakeDataProducerFactory for NameProducerFactory {
    fn name(&self) -> String {
        "Name".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.values_iter()
            .any(|value| matches!(value, DataValueRef::String(_) | DataValueRef::Null))
    }

    fn prompt(&self, _item: &DataValueItem) -> eyre::Result<Option<Box<dyn FakeDataProducer>>> {
        let type_options = NameStyle::VARIANTS.to_vec();
        let ty = match Select::new("What type of name would you like?", type_options)
            .prompt_skippable()?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(NameProducer { ty })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameProducer {
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
impl FakeDataProducer for NameProducer {
    fn produce_fake(&self, _original_value: DataValueRef<'_>) -> eyre::Result<DataValue> {
        let value = match &self.ty {
            NameStyle::Username => Username().fake(),
            NameStyle::FirstName => FirstName().fake(),
            NameStyle::LastName => LastName().fake(),
            NameStyle::Name => Name().fake(),
            NameStyle::NameWithTitle => NameWithTitle().fake(),
            NameStyle::Suffix => Suffix().fake(),
            NameStyle::Title => Title().fake(),
        };

        Ok(DataValue::String(value))
    }
}

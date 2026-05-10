use fake::{
    Fake,
    uuid::{UUIDv1, UUIDv3, UUIDv4, UUIDv5, UUIDv6, UUIDv7},
};
use inquire::Select;
use strum::{Display, VariantArray};
use uuid::Uuid;

use crate::{
    fake::{FakeDataProducer, FakeDataProducerFactory},
    json::{JsonPathItem, JsonValue},
};

pub struct UuidFakeDataFactory;

impl FakeDataProducerFactory for UuidFakeDataFactory {
    fn name(&self) -> String {
        "UUID".to_owned()
    }

    fn is_allowed_for(&self, item: &JsonPathItem) -> bool {
        matches!(item.value, JsonValue::String(_) | JsonValue::Null)
    }

    fn prompt(
        &self,
        item: &JsonPathItem,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let unit_options = UuidVersion::VARIANTS.to_vec();

        let target_uuid_version =
            original_value_version(&item.value).and_then(UuidVersion::equivalent);

        let target_uuid_version_index = target_uuid_version
            .and_then(|target_version| {
                unit_options
                    .iter()
                    .position(|value| target_version.eq(value))
            })
            .unwrap_or_default();

        let version = match Select::new("What's unit of lorem ipsum would you like?", unit_options)
            .with_starting_cursor(target_uuid_version_index)
            .prompt_skippable()?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(UuidFakeData { version })))
    }
}

#[derive(Display, VariantArray, Clone, Copy, PartialEq, Eq)]
pub enum UuidVersion {
    V1,
    V3,
    V4,
    V5,
    V6,
    V7,
}

impl UuidVersion {
    pub fn fake(&self) -> serde_json::Value {
        let value = match self {
            UuidVersion::V1 => UUIDv1.fake(),
            UuidVersion::V3 => UUIDv3.fake(),
            UuidVersion::V4 => UUIDv4.fake(),
            UuidVersion::V5 => UUIDv5.fake(),
            UuidVersion::V6 => UUIDv6.fake(),
            UuidVersion::V7 => UUIDv7.fake(),
        };

        serde_json::Value::String(value)
    }

    fn equivalent(other: uuid::Version) -> Option<UuidVersion> {
        Some(match other {
            uuid::Version::Mac => UuidVersion::V1,
            uuid::Version::Md5 => UuidVersion::V3,
            uuid::Version::Random => UuidVersion::V4,
            uuid::Version::Sha1 => UuidVersion::V5,
            uuid::Version::SortMac => UuidVersion::V6,
            uuid::Version::SortRand => UuidVersion::V7,

            _ => return None,
        })
    }
}

pub struct UuidFakeData {
    version: UuidVersion,
}

impl FakeDataProducer for UuidFakeData {
    fn produce_fake(&self, _original_value: &serde_json::Value) -> serde_json::Value {
        self.version.fake()
    }

    fn is_allowed_output(&self) -> bool {
        true
    }
}

fn original_value_version(value: &JsonValue) -> Option<uuid::Version> {
    let value = match value {
        JsonValue::String(value) => value,
        _ => return None,
    };

    let uuid: Uuid = value.parse().ok()?;
    uuid.get_version()
}

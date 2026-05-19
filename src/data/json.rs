use eyre::Context;
use thiserror::Error;

use super::key::PathKeyItem;
use crate::{
    data::{
        UpdateStructureData,
        key::PathKey,
        value::{DataValue, DataValueItem, DataValueNumber, DataValueNumberRef, DataValueRef},
    },
    fake::FakeDataProducerData,
};
use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
};

impl TryFrom<DataValue> for serde_json::Value {
    type Error = serde_json::Error;

    fn try_from(value: DataValue) -> Result<Self, Self::Error> {
        Ok(match value {
            DataValue::Number(value) => {
                let value = serde_json::Number::from_str(value.as_str())?;
                serde_json::Value::Number(value)
            }
            DataValue::String(value) => serde_json::Value::String(value),
            DataValue::Boolean(value) => serde_json::Value::Bool(value),
            DataValue::Null => serde_json::Value::Null,
        })
    }
}

#[derive(Debug, Error)]
#[error("unsupported json type for DataValue")]
pub struct UnsupportedJsonDataValue;

impl TryFrom<serde_json::Value> for DataValue {
    type Error = UnsupportedJsonDataValue;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::Null => DataValue::Null,
            serde_json::Value::Bool(value) => DataValue::Boolean(value),
            serde_json::Value::Number(value) => {
                DataValue::Number(DataValueNumber::new(value.to_string()))
            }
            serde_json::Value::String(value) => DataValue::String(value),

            _ => return Err(UnsupportedJsonDataValue),
        })
    }
}

impl<'a> TryFrom<&'a serde_json::Value> for DataValueRef<'a> {
    type Error = UnsupportedJsonDataValue;

    fn try_from(value: &'a serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::Null => DataValueRef::Null,
            serde_json::Value::Bool(value) => DataValueRef::Boolean(*value),
            serde_json::Value::Number(value) => {
                DataValueRef::Number(DataValueNumberRef::new(value.as_str()))
            }
            serde_json::Value::String(value) => DataValueRef::String(value.as_str()),
            _ => return Err(UnsupportedJsonDataValue),
        })
    }
}

/// Stack based iterator for walking a JSON structure
/// producing [DataValueItem]'s for each value in the JSON
struct JsonWalkIter<'a> {
    stack: VecDeque<WalkStackItem<'a>>,
}

struct WalkStackItem<'a> {
    key: Option<Arc<PathKey>>,
    value: &'a serde_json::Value,
}

impl<'a> JsonWalkIter<'a> {
    pub fn new(value: &'a serde_json::Value) -> eyre::Result<Self> {
        eyre::ensure!(
            matches!(
                value,
                serde_json::Value::Array(_) | serde_json::Value::Object(_)
            ),
            "must be provided either Array or Object as the root JSON value"
        );

        let mut stack = VecDeque::new();
        stack.push_back(WalkStackItem { value, key: None });
        Ok(Self { stack })
    }
}

impl<'a> Iterator for JsonWalkIter<'a> {
    type Item = DataValueItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.stack.pop_front()?;
            if let Ok(data_value_ref) = DataValueRef::try_from(item.value) {
                let key = item.key?;
                return Some(DataValueItem::new(key, data_value_ref));
            }

            match item.value {
                serde_json::Value::Array(values) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for value in values.iter().rev() {
                        let key = PathKey::new(item.key.clone(), PathKeyItem::Index);
                        self.stack.push_front(WalkStackItem {
                            key: Some(Arc::new(key)),
                            value,
                        });
                    }
                }
                serde_json::Value::Object(map) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for (key, value) in map.iter().rev() {
                        let key = PathKey::new(item.key.clone(), PathKeyItem::Key(key.to_owned()));
                        self.stack.push_front(WalkStackItem {
                            key: Some(Arc::new(key)),
                            value,
                        });
                    }
                }

                // This should never occur as DataValueRef::try_from handles all the other serde_json::Value types
                _ => unreachable!("unexpected json value encountered"),
            }
        }
    }
}

/// Walks the JSON structure providing a de-duplicated collection of [DataValueItem]'s
pub fn json_data_value_items(value: &serde_json::Value) -> eyre::Result<Vec<DataValueItem<'_>>> {
    let iter = JsonWalkIter::new(value)?;
    let mut existing_items = HashMap::new();
    let mut output = Vec::new();

    for item in iter {
        // Extend the value set for existing items
        if let Some(existing_index) = existing_items.get(&item.key) {
            let existing_item: &mut DataValueItem<'_> = &mut output[*existing_index];
            existing_item.values.extend(item.values);
            continue;
        }

        existing_items.insert(item.key.clone(), output.len());
        output.push(item);
    }

    Ok(output)
}

/// Stack based iterator for walking a JSON structure
/// producing [WalkStackItemMut]'s which provides mutable
/// access to the underlying JSON value to allow mutating
/// the JSON structure
struct JsonWalkIterMut<'a> {
    stack: VecDeque<WalkStackItemMut<'a>>,
}

struct WalkStackItemMut<'a> {
    key: Option<Arc<PathKey>>,
    value: &'a mut serde_json::Value,
}

impl<'a> JsonWalkIterMut<'a> {
    pub fn new(value: &'a mut serde_json::Value) -> eyre::Result<Self> {
        eyre::ensure!(
            matches!(
                value,
                serde_json::Value::Array(_) | serde_json::Value::Object(_)
            ),
            "must be provided either Array or Object as the root JSON value"
        );

        let mut stack = VecDeque::new();
        stack.push_back(WalkStackItemMut { value, key: None });
        Ok(Self { stack })
    }
}

impl<'a> Iterator for JsonWalkIterMut<'a> {
    type Item = WalkStackItemMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.stack.pop_front()?;

            match item.value {
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_)
                | serde_json::Value::String(_) => return Some(item),
                serde_json::Value::Array(values) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for value in values.iter_mut().rev() {
                        self.stack.push_front(WalkStackItemMut {
                            key: Some(Arc::new(PathKey::new(item.key.clone(), PathKeyItem::Index))),
                            value,
                        });
                    }
                }
                serde_json::Value::Object(map) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for (key, value) in map.iter_mut().rev() {
                        self.stack.push_front(WalkStackItemMut {
                            key: Some(Arc::new(PathKey::new(
                                item.key.clone(),
                                PathKeyItem::Key(key.to_owned()),
                            ))),
                            value,
                        });
                    }
                }
            }
        }
    }
}

/// Walks the JSON structure applying the updates
pub fn json_update_data(
    value: &mut serde_json::Value,
    data: &mut UpdateStructureData,
    producer_data: &mut FakeDataProducerData,
) -> eyre::Result<()> {
    let iter = JsonWalkIterMut::new(value)?;

    for item in iter {
        // Skip null if we are maintaining null
        if item.value.is_null() && data.config.maintain_null {
            continue;
        }

        match item.value {
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {
                let key = match &item.key {
                    Some(value) => value,
                    None => eyre::bail!("value item should always have a key"),
                };

                // Check for mapping to an existing generated value
                if let Some(output_override) = data.mapping.get(item.value) {
                    *item.value = output_override.clone();
                    continue;
                }

                let faker_data = data
                    .config
                    .mapping
                    .get(key)
                    .or(data.config.default.as_ref())
                    .ok_or(eyre::eyre!("item was missing from structure mapping"))?;

                let existing_value_ref = DataValueRef::try_from(&*item.value)?;

                let new_value = faker_data
                    .produce_fake(existing_value_ref, producer_data)
                    .context("failed to generate new value")?;

                let json_value: serde_json::Value = new_value.try_into()?;

                // Store the updated value
                if data.config.output.contains(key) {
                    let mapping = data.output_mapping.entry(key.clone()).or_default();
                    mapping.insert(item.value.clone(), json_value.clone());
                }

                // Map the value in the current mapping data if allowed to build internal mapping
                if data.config.internal_mapping {
                    data.mapping.insert(item.value.clone(), json_value.clone());
                }

                *item.value = json_value;
            }

            _ => eyre::bail!("unexpected item value type"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::data::{
        json::json_data_value_items,
        key::PathKey,
        value::{DataValueItem, DataValueRef},
    };
    use serde_json::json;
    use std::sync::Arc;

    fn test_path_key(key: &str) -> Arc<PathKey> {
        key.parse::<PathKey>()
            .map(Arc::new)
            .expect("invalid test path key")
    }

    /// Tests that a JSON structure can be successfully walked
    #[test]
    fn test_walk_json_structure() {
        let values = json!({
            "key": "value",
            "nested": {
                "a": "value_a",
                "b": "value_b"
            },
            "array": [
                {
                    "id": "test"
                },
                {
                    "id": "test_2",
                    "test": "array_test",
                }
            ]
        });
        let structure = json_data_value_items(&values).unwrap();

        assert_eq!(
            &structure,
            &[
                DataValueItem::new(test_path_key("key"), DataValueRef::String("value")),
                DataValueItem::new(test_path_key("nested.a"), DataValueRef::String("value_a")),
                DataValueItem::new(test_path_key("nested.b"), DataValueRef::String("value_b")),
                DataValueItem::new_many(
                    test_path_key("array.[index].id"),
                    vec![DataValueRef::String("test"), DataValueRef::String("test_2")]
                ),
                DataValueItem::new(
                    test_path_key("array.[index].test"),
                    DataValueRef::String("array_test")
                )
            ]
        )
    }

    /// Tests that when walking a JSON array keys that have
    /// already been visited previously will not appear
    /// multiple times
    #[test]
    fn test_walk_array_index_deduplication() {
        let values = json!([
            {
                "id": "test"
            },
            {
                "id": "test_2",
                "test": "array_test",
            }
        ]);
        let structure = json_data_value_items(&values).unwrap();
        assert_eq!(
            &structure,
            &[
                DataValueItem::new_many(
                    test_path_key("[index].id"),
                    vec![DataValueRef::String("test"), DataValueRef::String("test_2")]
                ),
                DataValueItem::new(
                    test_path_key("[index].test"),
                    DataValueRef::String("array_test")
                )
            ]
        )
    }
}

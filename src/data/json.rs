use eyre::Context;
use thiserror::Error;

use super::key::PathKeyItem;
use crate::data::{
    UpdateStructureData,
    key::PathKey,
    value::{DataValue, DataValueItem, DataValueRef},
};
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

impl From<DataValue> for serde_json::Value {
    fn from(value: DataValue) -> Self {
        match value {
            DataValue::Number(value) => serde_json::Value::Number(value),
            DataValue::String(value) => serde_json::Value::String(value),
            DataValue::Boolean(value) => serde_json::Value::Bool(value),
            DataValue::Null => serde_json::Value::Null,
        }
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
            serde_json::Value::Number(value) => DataValue::Number(value),
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
            serde_json::Value::Number(value) => DataValueRef::Number(value.clone()),
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
    type Item = DataValueItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.stack.pop_front()?;

            match item.value {
                serde_json::Value::Null => {
                    let key = item.key?;
                    return Some(DataValueItem {
                        key,
                        value: DataValue::Null,
                    });
                }
                serde_json::Value::Bool(value) => {
                    let key = item.key?;
                    return Some(DataValueItem {
                        key,
                        value: DataValue::Boolean(*value),
                    });
                }
                serde_json::Value::Number(value) => {
                    let key = item.key?;
                    return Some(DataValueItem {
                        key,
                        value: DataValue::Number(value.clone()),
                    });
                }
                serde_json::Value::String(value) => {
                    let key = item.key?;
                    return Some(DataValueItem {
                        key,
                        value: DataValue::String(value.clone()),
                    });
                }
                serde_json::Value::Array(values) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for value in values.iter().rev() {
                        self.stack.push_front(WalkStackItem {
                            key: Some(Arc::new(PathKey::new(item.key.clone(), PathKeyItem::Index))),
                            value,
                        });
                    }
                }
                serde_json::Value::Object(map) => {
                    // Push to the front in reverse order so we iterate in the same order
                    for (key, value) in map.iter().rev() {
                        self.stack.push_front(WalkStackItem {
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

/// Walks the JSON structure providing a de-duplicated collection of [DataValueItem]'s
pub fn json_data_value_items(value: &serde_json::Value) -> eyre::Result<Vec<DataValueItem>> {
    let iter = JsonWalkIter::new(value)?;
    let mut visited_keys = HashSet::new();
    let mut output = Vec::new();

    for item in iter {
        if visited_keys.contains(&item.key) {
            continue;
        }

        visited_keys.insert(item.key.clone());
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
) -> eyre::Result<()> {
    let iter = JsonWalkIterMut::new(value)?;

    for item in iter {
        match item.value {
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {
                let key = match &item.key {
                    Some(value) => value,
                    None => eyre::bail!("value item should always have a key"),
                };

                // Override from existing data
                if let Some(output_override) = data
                    .existing_output_mapping
                    .as_ref()
                    .and_then(|map| map.get(item.value))
                {
                    *item.value = output_override.clone();
                    continue;
                }

                let faker_data = data
                    .mappings
                    .get(key)
                    .ok_or(eyre::eyre!("item was missing from structure mapping"))?;

                let existing_value_ref = DataValueRef::try_from(&*item.value)?;

                let new_value = faker_data
                    .produce_fake(existing_value_ref)
                    .context("failed to generate new value")?;

                // Store the updated value
                if data.output_keys.contains(key) {
                    let mapping = data.output_mapping.entry(key.clone()).or_default();
                    let mapped_value = new_value.clone();

                    mapping.insert(item.value.clone(), mapped_value.into());
                }

                *item.value = new_value.into();
            }

            _ => eyre::bail!("unexpected item value type"),
        }
    }

    Ok(())
}

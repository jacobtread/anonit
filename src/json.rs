use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use eyre::Context;

use crate::{
    data::key::{PathKey, PathKeyItem},
    fake::FakeDataProducer,
};

#[derive(Debug, Clone)]
pub struct JsonPathItem {
    pub path_key: Arc<PathKey>,
    pub value: JsonValue,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum JsonValue {
    Number(serde_json::Number),
    String(String),
    Boolean(bool),
    Null,
}

/// Walks the provided JSON structure determining the paths and all available
/// keys that can be used for redaction
pub fn build_json_structure(value: &serde_json::Value) -> eyre::Result<Vec<JsonPathItem>> {
    match value {
        serde_json::Value::Array(value) => {
            let mut items: Vec<JsonPathItem> = Vec::new();
            walk_json_array(value, None, &mut items)?;
            Ok(items)
        }
        serde_json::Value::Object(value) => {
            let mut items: Vec<JsonPathItem> = Vec::new();
            walk_json_object(value, None, &mut items)?;
            Ok(items)
        }
        _ => eyre::bail!("json structure must start with either an array or an object"),
    }
}

/// Ensures only the first instance of array index items are maintained
pub fn deduplicate_json_structure(structure: &mut Vec<JsonPathItem>) {
    let mut visited_hashes = HashSet::new();

    structure.retain(|item| {
        if visited_hashes.contains(&item.path_key) {
            return false;
        }

        visited_hashes.insert(item.path_key.clone());
        true
    });
}

pub fn walk_json_field(
    value: &serde_json::Value,
    key: Arc<PathKey>,
    output: &mut Vec<JsonPathItem>,
) -> eyre::Result<()> {
    match value {
        serde_json::Value::Null => {
            output.push(JsonPathItem {
                path_key: key,
                value: JsonValue::Null,
            });
            Ok(())
        }
        serde_json::Value::Bool(value) => {
            output.push(JsonPathItem {
                path_key: key,
                value: JsonValue::Boolean(*value),
            });
            Ok(())
        }
        serde_json::Value::Number(number) => {
            output.push(JsonPathItem {
                path_key: key,
                value: JsonValue::Number(number.clone()),
            });
            Ok(())
        }
        serde_json::Value::String(value) => {
            output.push(JsonPathItem {
                path_key: key,
                value: JsonValue::String(value.to_string()),
            });
            Ok(())
        }
        serde_json::Value::Array(value) => walk_json_array(value, Some(key), output),
        serde_json::Value::Object(value) => walk_json_object(value, Some(key), output),
    }
}

fn walk_json_array(
    value: &[serde_json::Value],
    key: Option<Arc<PathKey>>,
    output: &mut Vec<JsonPathItem>,
) -> eyre::Result<()> {
    for value in value {
        let key = Arc::new(PathKey::new(key.clone(), PathKeyItem::Index));

        walk_json_field(value, key, output)?;
    }

    Ok(())
}

fn walk_json_object(
    value: &serde_json::Map<String, serde_json::Value>,
    key: Option<Arc<PathKey>>,
    output: &mut Vec<JsonPathItem>,
) -> eyre::Result<()> {
    for (object_key, value) in value {
        let key = Arc::new(PathKey::new(
            key.clone(),
            PathKeyItem::Key(object_key.to_string()),
        ));

        walk_json_field(value, key, output)?;
    }

    Ok(())
}

pub type OutputMappingMap = HashMap<Arc<PathKey>, HashMap<serde_json::Value, serde_json::Value>>;

pub struct UpdateJsonData {
    pub mappings: HashMap<Arc<PathKey>, Box<dyn FakeDataProducer>>,
    pub output_keys: HashSet<Arc<PathKey>>,
    pub output_mapping: OutputMappingMap,
    pub existing_output_mapping: Option<HashMap<serde_json::Value, serde_json::Value>>,
}

pub fn update_json_structure(
    value: &serde_json::Value,
    data: &mut UpdateJsonData,
) -> eyre::Result<serde_json::Value> {
    match value {
        serde_json::Value::Array(value) => walk_json_array_update(value, None, data),
        serde_json::Value::Object(value) => walk_json_object_update(value, None, data),
        _ => eyre::bail!("json structure must start with either an array or an object"),
    }
}

pub fn walk_json_field_update(
    value: &serde_json::Value,
    key: Arc<PathKey>,
    data: &mut UpdateJsonData,
) -> eyre::Result<serde_json::Value> {
    match value {
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {
            // Override from existing data
            if let Some(output_override) = data
                .existing_output_mapping
                .as_ref()
                .and_then(|map| map.get(value))
            {
                return Ok(output_override.clone());
            }

            let faker_data = data
                .mappings
                .get(&key)
                .ok_or(eyre::eyre!("item was missing from structure mapping"))?;
            let new_value = faker_data
                .produce_fake(value)
                .context("failed to generate new value")?;

            // Store the updated value
            if data.output_keys.contains(&key) {
                let mapping = data.output_mapping.entry(key.clone()).or_default();
                mapping.insert(value.clone(), new_value.clone());
            }

            Ok(new_value)
        }
        serde_json::Value::Array(value) => walk_json_array_update(value, Some(key), data),
        serde_json::Value::Object(value) => walk_json_object_update(value, Some(key), data),
    }
}

fn walk_json_array_update(
    value: &[serde_json::Value],
    key: Option<Arc<PathKey>>,
    data: &mut UpdateJsonData,
) -> eyre::Result<serde_json::Value> {
    let mut values = Vec::new();

    for value in value {
        let key = Arc::new(PathKey::new(key.clone(), PathKeyItem::Index));

        let value = walk_json_field_update(value, key, data)?;
        values.push(value)
    }

    Ok(serde_json::Value::Array(values))
}

fn walk_json_object_update(
    value: &serde_json::Map<String, serde_json::Value>,
    key: Option<Arc<PathKey>>,
    data: &mut UpdateJsonData,
) -> eyre::Result<serde_json::Value> {
    let mut map = serde_json::Map::new();

    for (object_key, value) in value {
        let key = Arc::new(PathKey::new(
            key.clone(),
            PathKeyItem::Key(object_key.to_string()),
        ));

        let new_value = walk_json_field_update(value, key, data)?;
        map.insert(object_key.clone(), new_value);
    }

    Ok(serde_json::Value::Object(map))
}

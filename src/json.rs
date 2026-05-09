use std::{
    collections::HashSet,
    fmt::{Debug, Display, Write},
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum PathKeyItem {
    Index(usize),
    Key(String),
}

impl PathKeyItem {
    fn hash_excluding_index<H: Hasher>(&self, hasher: &mut H) {
        match self {
            PathKeyItem::Index(_) => "index".hash(hasher),
            PathKeyItem::Key(key) => key.hash(hasher),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PathKey {
    /// The parent key
    parent: Option<Arc<PathKey>>,
    /// The current key segment
    item: PathKeyItem,
}

impl PathKey {
    fn hash_excluding_index<H: Hasher>(&self, hasher: &mut H) {
        if let Some(parent) = self.parent.as_ref() {
            parent.hash_excluding_index(hasher);
        }

        self.item.hash_excluding_index(hasher);
    }
}

impl Display for PathKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parent: Option<&Arc<PathKey>> = self.parent.as_ref();
        let mut stack = Vec::new();

        stack.push(&self.item);
        while let Some(key) = parent {
            stack.push(&key.item);
            parent = key.parent.as_ref();
        }

        stack.reverse();

        let stack_len = stack.len();

        for (index, item) in stack.into_iter().enumerate() {
            match &item {
                PathKeyItem::Index(index) => write!(f, "[{index}]")?,
                PathKeyItem::Key(key) => f.write_str(key)?,
            }

            if index < stack_len - 1 {
                f.write_char('.')?;
            }
        }

        Ok(())
    }
}

impl Debug for PathKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <PathKey as Display>::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct JsonPathItem {
    pub path_key: Arc<PathKey>,
    pub value: JsonValue,
}

#[derive(Debug, Clone)]
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
        let mut hasher = DefaultHasher::default();
        item.path_key.hash_excluding_index(&mut hasher);
        let hash = hasher.finish();

        if visited_hashes.contains(&hash) {
            return false;
        }

        visited_hashes.insert(hash);
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
    for (index, value) in value.iter().enumerate() {
        let key = Arc::new(PathKey {
            parent: key.clone(),
            item: PathKeyItem::Index(index),
        });

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
        let key = Arc::new(PathKey {
            parent: key.clone(),
            item: PathKeyItem::Key(object_key.to_string()),
        });

        walk_json_field(value, key, output)?;
    }

    Ok(())
}

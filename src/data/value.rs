use std::sync::Arc;

use crate::data::key::PathKey;

#[derive(Debug, Clone)]
pub enum DataValue {
    Number(serde_json::Number),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub enum DataValueRef<'a> {
    Number(serde_json::Number),
    String(&'a str),
    Boolean(bool),
    Null,
}

impl<'a> From<DataValueRef<'a>> for DataValue {
    fn from(value: DataValueRef<'a>) -> DataValue {
        match value {
            DataValueRef::Number(value) => DataValue::Number(value),
            DataValueRef::String(value) => DataValue::String(value.to_owned()),
            DataValueRef::Boolean(value) => DataValue::Boolean(value),
            DataValueRef::Null => DataValue::Null,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataValueItem {
    pub key: Arc<PathKey>,
    pub value: DataValue,
}

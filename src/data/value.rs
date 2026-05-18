use std::sync::Arc;

use crate::data::key::PathKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataValueNumber(String);

impl DataValueNumber {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<DataValueNumber> for String {
    fn from(value: DataValueNumber) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataValueNumberRef<'a>(&'a str);

impl<'a> DataValueNumberRef<'a> {
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataValue {
    Number(DataValueNumber),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataValueRef<'a> {
    Number(DataValueNumberRef<'a>),
    String(&'a str),
    Boolean(bool),
    Null,
}

impl<'a> From<DataValueRef<'a>> for DataValue {
    fn from(value: DataValueRef<'a>) -> DataValue {
        match value {
            DataValueRef::Number(value) => DataValue::Number(DataValueNumber(value.0.to_owned())),
            DataValueRef::String(value) => DataValue::String(value.to_owned()),
            DataValueRef::Boolean(value) => DataValue::Boolean(value),
            DataValueRef::Null => DataValue::Null,
        }
    }
}

impl<'a> DataValueRef<'a> {
    pub fn is_number_or_null(&self) -> bool {
        matches!(self, DataValueRef::Number(_) | DataValueRef::Null)
    }

    pub fn is_string_or_null(&self) -> bool {
        matches!(self, DataValueRef::String(_) | DataValueRef::Null)
    }

    pub fn is_bool_or_null(&self) -> bool {
        matches!(self, DataValueRef::Boolean(_) | DataValueRef::Null)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataValueItem<'a> {
    pub key: Arc<PathKey>,
    pub values: Vec<DataValueRef<'a>>,
}

impl<'a> DataValueItem<'a> {
    pub fn new(key: Arc<PathKey>, value: DataValueRef<'a>) -> Self {
        Self {
            key,
            values: vec![value],
        }
    }

    #[allow(unused)]
    pub fn new_many(key: Arc<PathKey>, values: Vec<DataValueRef<'a>>) -> Self {
        Self { key, values }
    }

    pub fn values_iter<'d>(&'d self) -> std::slice::Iter<'d, DataValueRef<'a>> {
        self.values.iter()
    }

    pub fn is_any_string_or_null(&self) -> bool {
        self.values_iter().any(|value| value.is_string_or_null())
    }

    pub fn is_any_number_or_null(&self) -> bool {
        self.values_iter().any(|value| value.is_number_or_null())
    }
}

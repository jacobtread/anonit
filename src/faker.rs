use std::fmt::Display;

use fake::{
    Fake,
    faker::{boolean::en::Boolean, lorem::zh_tw::Paragraph},
};
use inquire::Select;

use crate::json::{self, JsonPathItem};

pub struct ItemWithFaker {
    pub item: JsonPathItem,
    pub faker_type: FakerType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerType {
    // Type that applies no fake value
    Ignore,
    Boolean,
    LoremIpsum,
    Integer(FakerIntegerType),
    Float(FakerFloatType),
    Ip(FakerIpType),
    Name(FakerNameType),
    DateTime(FakerDateTimeType),
    User(FakerUserType),
    File(FakerFileType),
}

impl FakerType {
    pub fn fake(&self, original_value: &serde_json::Value) -> serde_json::Value {
        match self {
            FakerType::Ignore => original_value.clone(),
            FakerType::Boolean => {
                let value = Boolean(1).fake();
                serde_json::Value::Bool(value)
            }
            FakerType::LoremIpsum => {
                let value = Paragraph(0..50).fake();
                serde_json::Value::String(value)
            }
            FakerType::Integer(faker_integer_type) => todo!(),
            FakerType::Float(faker_float_type) => todo!(),
            FakerType::Ip(faker_ip_type) => todo!(),
            FakerType::Name(faker_name_type) => todo!(),
            FakerType::DateTime(faker_date_time_type) => todo!(),
            FakerType::User(faker_user_type) => todo!(),
            FakerType::File(faker_file_type) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerFileType {
    FileName,
    FileExtension,
    FilePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerDateTimeType {
    Date,
    DateTime,
    Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerUserType {
    Email,
    Username,
    Password,
    PhoneNumber,
    Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerNameType {
    FirstName,
    LastName,
    Name,
    NameWithTitle,
    Suffix,
    Title,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerIpType {
    Ipv4,
    Ipv6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerIntegerType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerFloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakerTypeFilter {
    Number,
    String,
    Boolean,
}

static ALL_FAKER_TYPES: &[FakerTypeFilter] = &[
    FakerTypeFilter::Number,
    FakerTypeFilter::String,
    FakerTypeFilter::Boolean,
];

static FAKER_TYPE_ITEMS: &[FakerItem] = &[
    FakerItem {
        name: "Ignore",
        filter: ALL_FAKER_TYPES,
        ty: FakerType::Ignore,
    },
    FakerItem {
        name: "Boolean",
        filter: &[FakerTypeFilter::Boolean],
        ty: FakerType::Boolean,
    },
    FakerItem {
        name: "Lorem Ipsum",
        filter: &[FakerTypeFilter::String],
        ty: FakerType::LoremIpsum,
    },
    FakerItem {
        name: "Integer (64bit)",
        filter: &[FakerTypeFilter::Number],
        ty: FakerType::Integer(FakerIntegerType::I64),
    },
];

pub struct FakerItem {
    pub name: &'static str,
    pub filter: &'static [FakerTypeFilter],
    pub ty: FakerType,
}

impl Display for FakerItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

pub fn prompt_item_faker_type(item: &JsonPathItem) -> eyre::Result<FakerType> {
    let faker_item_type = match &item.value {
        json::JsonValue::Number(_) => Some(FakerTypeFilter::Number),
        json::JsonValue::String(_) => Some(FakerTypeFilter::String),
        json::JsonValue::Boolean(_) => Some(FakerTypeFilter::Boolean),
        json::JsonValue::Null => None,
    };

    let items: Vec<&FakerItem> = FAKER_TYPE_ITEMS
        .iter()
        .filter(|item| {
            if faker_item_type.is_none() {
                return true;
            }

            let item_type = match faker_item_type.as_ref() {
                Some(value) => value,
                None => return true,
            };

            item.filter.contains(item_type)
        })
        .collect();

    let key = item.path_key.to_string();
    let answer = Select::new(&format!("What type should \"{key}\" be?"), items).prompt()?;
    Ok(answer.ty)
}

fn suggest_item_type(item: &JsonPathItem) -> Option<FakerType> {
    match &item.value {
        json::JsonValue::Number(number) => {
            if number.is_f64() {
                Some(FakerType::Float(FakerFloatType::F64))
            } else if number.is_i64() {
                Some(FakerType::Integer(FakerIntegerType::I64))
            } else if number.is_u64() {
                Some(FakerType::Integer(FakerIntegerType::U64))
            } else {
                Some(FakerType::Integer(FakerIntegerType::I64))
            }
        }
        json::JsonValue::String(_value) => {
            // TODO: Match item types to guess
            Some(FakerType::LoremIpsum)
        }
        json::JsonValue::Boolean(_) => Some(FakerType::Boolean),
        json::JsonValue::Null => None,
    }
}

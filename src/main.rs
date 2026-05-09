use std::{fs::File, io::read_to_string};

use crate::json::{JsonPathItem, build_json_structure, deduplicate_json_structure};

mod json;

fn main() -> eyre::Result<()> {
    let file = read_to_string(File::open("./private/input.json")?)?;
    let parsed = serde_json::from_str(&file)?;
    let mut structure = build_json_structure(&parsed)?;
    deduplicate_json_structure(&mut structure);

    dbg!(structure);

    Ok(())
}

enum FakerType {
    // Type that applies no fake value
    Leave,
    Address,
    Boolean,
    PhoneNumber,
    Date,
    DateTime,
    Duration,
    Email,
    FirstName,
    LastName,
    Name,
    NameWithTitle,
    Suffix,
    Title,
    Username,
    Password,
    Ipv4,
    Ipv6,
    FileName,
    FileExtension,
    FilePath,
    Text,
    Integer(FakerIntegerType),
    Float(FakerFloatType),
}

enum FakerIntegerType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

enum FakerFloatType {
    F32,
    F64,
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
            Some(FakerType::Text)
        }
        json::JsonValue::Boolean(_) => Some(FakerType::Boolean),
        json::JsonValue::Null => None,
    }
}

fn prompt_structure_types() {}

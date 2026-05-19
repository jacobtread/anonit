use crate::data::{UpdateStructureData, json::json_update_data};

pub mod config;
pub mod ctx;
pub mod data;
pub mod fake;
mod prompt_utils;

/// Process the `input_data` JSON data using the provided `data` update
/// configuration and produce the updated JSON output
pub fn process_json_file(
    input_data: serde_json::Value,
    data: &mut UpdateStructureData,
) -> eyre::Result<serde_json::Value> {
    let mut output = input_data.clone();
    json_update_data(&mut output, data)?;
    Ok(output)
}

use rand::TryRng;

use crate::{
    data::{UpdateStructureData, json::json_update_data},
    fake::FakeDataProducerData,
};

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

    // Generate a seed using the thread rng to share between our rng and rng_08
    let mut seed = [0u8; 32];
    rand::rng().try_fill_bytes(seed.as_mut())?;

    let rng = <rand::rngs::StdRng as rand::SeedableRng>::from_seed(seed);
    let rng_08 = <rand08::rngs::StdRng as rand08::SeedableRng>::from_seed(seed);

    let mut producer_data = FakeDataProducerData {
        rng,
        rng_08,
        ctx: Default::default(),
    };

    json_update_data(&mut output, data, &mut producer_data)?;
    Ok(output)
}

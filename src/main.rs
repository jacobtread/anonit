use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::read_to_string,
};

use inquire::prompt_confirmation;

use crate::{
    fake::{fake_data_registry, prompt_fake_data_type},
    json::{build_json_structure, deduplicate_json_structure, update_json_structure},
};

mod fake;
mod json;
mod prompt_utils;

fn main() -> eyre::Result<()> {
    let file = read_to_string(File::open("./private/input.json")?)?;
    let parsed = serde_json::from_str(&file)?;
    let mut structure = build_json_structure(&parsed)?;
    deduplicate_json_structure(&mut structure);

    let registry = fake_data_registry();

    let mut faker_data = HashMap::new();
    let mut output_keys = HashSet::new();

    for item in structure {
        let producer = prompt_fake_data_type(&registry, &item)?.ok_or(eyre::eyre!(
            "todo: handle cancelling to allow the user to try again"
        ))?;

        let key_hash = item.path_key.hashed_excluding_index();

        if producer.is_allowed_output() {
            let key = item.path_key.to_string();
            if prompt_confirmation(format!(
                "Do you want to create an output mapping for {key}?"
            ))? {
                output_keys.insert(key_hash);
            }
        }

        faker_data.insert(item.path_key.hashed_excluding_index(), producer);
    }

    let mut output_mapping: HashMap<String, HashMap<serde_json::Value, serde_json::Value>> =
        HashMap::new();
    let output = update_json_structure(&parsed, &faker_data, &output_keys, &mut output_mapping)?;
    let serialized = serde_json::to_string_pretty(&output)?;
    let serialized_mapping = serde_json::to_string_pretty(&output_mapping)?;

    println!("{serialized}\n");
    println!("{serialized_mapping}");

    Ok(())
}

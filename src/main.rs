use std::{collections::HashMap, fs::File, io::read_to_string};

use crate::{
    fake::{fake_data_registry, prompt_fake_data_type},
    faker::{ItemWithFaker, prompt_item_faker_type},
    json::{build_json_structure, deduplicate_json_structure, update_json_structure},
};

mod fake;
mod faker;
mod json;
mod prompt_utils;

fn main() -> eyre::Result<()> {
    let file = read_to_string(File::open("./private/input.json")?)?;
    let parsed = serde_json::from_str(&file)?;
    let mut structure = build_json_structure(&parsed)?;
    deduplicate_json_structure(&mut structure);

    let registry = fake_data_registry();

    let mut faker_data = HashMap::new();
    for item in structure {
        let producer = prompt_fake_data_type(&registry, &item)?.ok_or(eyre::eyre!(
            "todo: handle cancelling to allow the user to try again"
        ))?;

        faker_data.insert(item.path_key.hashed_excluding_index(), producer);
    }

    let output = update_json_structure(&parsed, &faker_data)?;
    let serialized = serde_json::to_string_pretty(&output)?;

    println!("{serialized}");

    Ok(())
}

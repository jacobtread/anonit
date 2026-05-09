use std::{collections::HashMap, fs::File, io::read_to_string};

use crate::{
    faker::{ItemWithFaker, prompt_item_faker_type},
    json::{build_json_structure, deduplicate_json_structure, update_json_structure},
};

mod faker;
mod json;

fn main() -> eyre::Result<()> {
    let file = read_to_string(File::open("./private/input.json")?)?;
    let parsed = serde_json::from_str(&file)?;
    let mut structure = build_json_structure(&parsed)?;
    deduplicate_json_structure(&mut structure);

    let mut faker_data = HashMap::new();
    for item in structure {
        let faker_type = prompt_item_faker_type(&item)?;
        faker_data.insert(
            item.path_key.hashed_excluding_index(),
            ItemWithFaker { item, faker_type },
        );
    }

    let output = update_json_structure(&parsed, &faker_data)?;
    let serialized = serde_json::to_string_pretty(&output)?;

    println!("{serialized}");

    Ok(())
}

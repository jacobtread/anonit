use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
};

use clap::Parser;
use eyre::Context;
use inquire::prompt_confirmation;

use crate::{
    fake::{fake_data_registry, prompt_fake_data_type},
    json::{
        OutputMappingMap, UpdateJsonData, build_json_structure, deduplicate_json_structure,
        update_json_structure,
    },
};

mod fake;
mod json;
mod prompt_utils;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file to process
    #[arg(short, long)]
    input: PathBuf,

    /// Optional mapping file from a previous run to use for
    /// keeping redacted IDs consistent across files
    #[arg(long)]
    input_mapping: Option<PathBuf>,

    /// Optional pre-made configuration file to decide how
    /// fields are redacted instead of prompting the user
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Optional output file to store the processed file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Optional output file to store generated mappings from
    /// pre-redacted field values to the post redacted values
    /// for use with redacts that need to have consistent IDs
    ///
    /// Omitting an output file will use stdout
    #[arg(long)]
    output_mapping: Option<PathBuf>,

    /// Optional output file to store the generated
    #[arg(long)]
    config_output: Option<PathBuf>,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let input_data: serde_json::Value = {
        let file = File::open(args.input).context("failed to open input file")?;
        serde_json::from_reader(file).context("failed to read / parse file")?
    };

    let input_mapping_data: Option<OutputMappingMap> = match args.input_mapping {
        Some(input_mapping) => {
            let file = File::open(input_mapping).context("failed to open input file")?;
            Some(serde_json::from_reader(file).context("failed to read / parse file")?)
        }
        None => None,
    };

    let flat_input_mapping_data: Option<HashMap<serde_json::Value, serde_json::Value>> =
        input_mapping_data.map(|mapping| {
            mapping
                .values()
                .flatten()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        });

    // TODO: Config file
    let _config: Option<serde_json::Value> = match args.config {
        Some(config) => {
            let file = File::open(config).context("failed to open input file")?;
            Some(serde_json::from_reader(file).context("failed to read / parse file")?)
        }
        None => None,
    };

    let mut structure = build_json_structure(&input_data)?;
    deduplicate_json_structure(&mut structure);

    let registry = fake_data_registry();

    let mut faker_data = HashMap::new();
    let mut output_keys = HashSet::new();

    for item in structure {
        let producer = prompt_fake_data_type(&registry, &item)?.ok_or(eyre::eyre!(
            "todo: handle cancelling to allow the user to try again"
        ))?;

        if producer.is_allowed_output() {
            let key = item.path_key.to_string();
            if prompt_confirmation(format!(
                "Do you want to create an output mapping for {key}?"
            ))? {
                output_keys.insert(item.path_key.clone());
            }
        }

        faker_data.insert(item.path_key.clone(), producer);
    }

    let output_mapping: OutputMappingMap = HashMap::new();

    let mut data = UpdateJsonData {
        mappings: faker_data,
        output_keys,
        output_mapping,
        existing_output_mapping: flat_input_mapping_data,
    };

    let output = update_json_structure(&input_data, &mut data)?;

    let serialized = serde_json::to_string_pretty(&output)?;
    if let Some(output) = args.output {
        let mut file = File::create(output).context("failed to open output file")?;
        file.write_all(serialized.as_bytes())
            .context("failed to write output")?;
        file.flush().context("failed to flush file")?;
    } else {
        println!("{serialized}");
        println!();
    }

    let serialized_mapping = serde_json::to_string_pretty(&data.output_mapping)?;
    if let Some(output_mapping) = args.output_mapping {
        let mut file = File::create(output_mapping).context("failed to open output file")?;
        file.write_all(serialized_mapping.as_bytes())
            .context("failed to write output")?;
        file.flush().context("failed to flush file")?;
    } else {
        println!("{serialized_mapping}");
    }

    Ok(())
}

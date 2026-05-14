use crate::{
    data::{
        OutputMappingMap, UpdateStructureData,
        json::{json_data_value_items, json_update_data},
        key::PathKey,
        value::DataValueItem,
    },
    fake::{FakeDataProducer, fake_data_registry, prompt_fake_data_type},
};
use clap::Parser;
use eyre::Context;
use inquire::prompt_confirmation;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::Arc,
};

mod data;
mod fake;
mod prompt_utils;

/// Data anonymizing tool.
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
    ///
    /// Omitting an output file will print the output to
    /// stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Optional output file to store generated mappings from
    /// pre-redacted field values to the post redacted values
    /// for use with redacts that need to have consistent IDs
    #[arg(long)]
    output_mapping: Option<PathBuf>,

    /// Optional output file to store the generated
    #[arg(long)]
    config_output: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    mapping: HashMap<Arc<PathKey>, Box<dyn FakeDataProducer>>,
    output: HashSet<Arc<PathKey>>,
}

/// Prompt the user to configure based on the structure
fn prompt_config(structure: &[DataValueItem]) -> eyre::Result<Config> {
    let registry = fake_data_registry();
    let mut mapping = HashMap::new();
    let mut output = HashSet::new();

    for item in structure {
        loop {
            let producer = match prompt_fake_data_type(&registry, item)? {
                Some(value) => value,
                None => continue,
            };

            // For keys that support outputting a mapping prompt the user if they want to
            if producer.is_allowed_output() {
                let key = item.key.to_string();
                if prompt_confirmation(format!(
                    "Do you want to create an output mapping for {key}?"
                ))? {
                    output.insert(item.key.clone());
                }
            }

            mapping.insert(item.key.clone(), producer);
            break;
        }
    }

    Ok(Config { mapping, output })
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
        input_mapping_data.map(|mapping| mapping.into_values().flatten().collect());

    let config: Option<Config> = match args.config {
        Some(config) => {
            let file = File::open(config).context("failed to open input file")?;
            Some(serde_json::from_reader(file).context("failed to read / parse file")?)
        }
        None => None,
    };

    let structure = json_data_value_items(&input_data)?;

    let config = match config {
        Some(config) => config,
        None => prompt_config(&structure)?,
    };

    let output_mapping: OutputMappingMap = HashMap::new();
    if let Some(config_output) = args.config_output {
        let serialized_config = serde_json::to_string_pretty(&config)?;
        let mut file = File::create(config_output).context("failed to open output file")?;
        file.write_all(serialized_config.as_bytes())
            .context("failed to write output")?;
        file.flush().context("failed to flush file")?;
    }

    let mut data = UpdateStructureData {
        mappings: config.mapping,
        output_keys: config.output,
        output_mapping,
        existing_output_mapping: flat_input_mapping_data,
    };

    let mut output = input_data.clone();
    json_update_data(&mut output, &mut data)?;

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
    }

    Ok(())
}

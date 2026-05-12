use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{data::key::PathKey, fake::FakeDataProducer};

pub mod json;
pub mod key;
pub mod value;

pub type OutputMappingMap = HashMap<Arc<PathKey>, HashMap<serde_json::Value, serde_json::Value>>;

pub struct UpdateStructureData {
    pub mappings: HashMap<Arc<PathKey>, Box<dyn FakeDataProducer>>,
    pub output_keys: HashSet<Arc<PathKey>>,
    pub output_mapping: OutputMappingMap,
    pub existing_output_mapping: Option<HashMap<serde_json::Value, serde_json::Value>>,
}

use std::{collections::HashMap, sync::Arc};

use crate::{config::Config, ctx::ContextData, data::key::PathKey};

pub mod json;
pub mod key;
pub mod value;

pub type OutputMappingMap = HashMap<Arc<PathKey>, HashMap<serde_json::Value, serde_json::Value>>;

pub struct UpdateStructureData {
    /// Configuration
    pub config: Config,

    /// Mapping data to use for values that should be replaced
    /// with a specific value if found
    pub mapping: HashMap<serde_json::Value, serde_json::Value>,

    /// Producer context data
    pub ctx: ContextData,

    /// Mapping data produced from processing the structure
    pub output_mapping: OutputMappingMap,
}

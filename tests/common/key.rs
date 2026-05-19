use anonit::data::key::PathKey;
use std::sync::Arc;

/// Helper for tests to create a key easily
pub fn path_key(key: &str) -> Arc<PathKey> {
    key.parse::<PathKey>()
        .map(Arc::new)
        .expect("invalid test path key")
}

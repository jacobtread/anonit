use std::{
    fmt::{Debug, Display, Write},
    str::FromStr,
    sync::Arc,
};

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Segment of a path key, index here intentionally omits the
/// index value in order to make all indexes of an array equivalent
/// in hash for matching
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum PathKeyItem {
    Index,
    Key(String),
}

/// Key for representing the path to an item in a data structure
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PathKey {
    /// The parent key
    parent: Option<Arc<PathKey>>,
    /// The current key segment
    item: PathKeyItem,
}

impl PathKey {
    pub fn new(parent: Option<Arc<PathKey>>, item: PathKeyItem) -> Self {
        Self { parent, item }
    }
}

impl FromIterator<PathKeyItem> for Option<PathKey> {
    fn from_iter<T: IntoIterator<Item = PathKeyItem>>(iter: T) -> Self {
        let mut current: Option<PathKey> = None;

        for item in iter {
            current = Some(PathKey {
                parent: current.map(Arc::new),
                item,
            });
        }

        current
    }
}

impl Serialize for PathKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PathKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PathKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PathKeyParseError {
    #[error("dangling escape sequence")]
    DanglingEscape,

    #[error("invalid escape sequence: {0}")]
    InvalidEscape(char),
}

impl PathKey {
    const ESCAPED_CHARS: &[char] = &['\\', '.', '[', ']'];

    /// Parses the provided input into path key segments while handling escape codes
    fn parse_escaped_segments(input: &str) -> Result<Vec<String>, PathKeyParseError> {
        let mut segments = Vec::<String>::new();
        let mut current = String::new();

        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '.' => {
                    segments.push(std::mem::take(&mut current));
                }

                '\\' => {
                    let escaped = chars.next().ok_or(PathKeyParseError::DanglingEscape)?;
                    if !Self::ESCAPED_CHARS.contains(&escaped) {
                        return Err(PathKeyParseError::InvalidEscape(escaped));
                    }

                    current.push(escaped);
                }

                _ => current.push(ch),
            }
        }

        segments.push(current);
        Ok(segments)
    }
}

impl FromStr for PathKey {
    type Err = PathKeyParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // An empty string is equivalent to referencing an empty root key in a JSON file
        // i.e { "": <-- this one "" }
        if input.is_empty() {
            return Ok(PathKey::new(None, PathKeyItem::Key("".into())));
        }

        let segments = PathKey::parse_escaped_segments(input)?;

        let mut current: Option<PathKey> = None;

        for segment in segments {
            let item = if segment == "[index]" {
                PathKeyItem::Index
            } else {
                PathKeyItem::Key(segment)
            };

            current = Some(PathKey {
                parent: current.map(Arc::new),
                item,
            });
        }

        Ok(match current {
            Some(current) => current,
            None => PathKey::new(None, PathKeyItem::Key("".into())),
        })
    }
}

impl Display for PathKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = Vec::<&PathKeyItem>::new();

        let mut current = Some(self);

        while let Some(key) = current {
            stack.push(&key.item);
            current = key.parent.as_deref();
        }

        stack.reverse();

        let last_index = stack.len() - 1;
        for (i, item) in stack.iter().enumerate() {
            match item {
                PathKeyItem::Index => {
                    f.write_str("[index]")?;
                }

                PathKeyItem::Key(key) => {
                    for ch in key.chars() {
                        if Self::ESCAPED_CHARS.contains(&ch) {
                            f.write_char('\\')?;
                        }

                        f.write_char(ch)?;
                    }
                }
            }

            if i < last_index {
                f.write_char('.')?;
            }
        }

        Ok(())
    }
}

impl Debug for PathKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <PathKey as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::data::key::{PathKey, PathKeyItem, PathKeyParseError};

    /// Test utility for generating a path key from a collection of path items
    fn test_path_key(items: impl IntoIterator<Item = PathKeyItem>) -> Arc<PathKey> {
        let mut current: Option<Arc<PathKey>> = None;

        for item in items {
            current = Some(Arc::new(PathKey {
                parent: current,
                item,
            }));
        }

        current.unwrap()
    }

    /// Tests that keys are displayed correctly
    #[test]
    fn test_key_display() {
        let data = &[
            (test_path_key([PathKeyItem::Key("Key".into())]), "Key"),
            (test_path_key([PathKeyItem::Index]), "[index]"),
            (
                test_path_key([PathKeyItem::Index, PathKeyItem::Key("Nested".into())]),
                "[index].Nested",
            ),
            (
                test_path_key([
                    PathKeyItem::Key("Test".into()),
                    PathKeyItem::Index,
                    PathKeyItem::Key("Nested".into()),
                ]),
                "Test.[index].Nested",
            ),
            (
                test_path_key([
                    PathKeyItem::Key("Test".into()),
                    PathKeyItem::Index,
                    PathKeyItem::Key("Nested".into()),
                    PathKeyItem::Key("Deep".into()),
                ]),
                "Test.[index].Nested.Deep",
            ),
            (
                test_path_key([
                    PathKeyItem::Index,
                    PathKeyItem::Index,
                    PathKeyItem::Key("Deep".into()),
                ]),
                "[index].[index].Deep",
            ),
            (
                test_path_key([PathKeyItem::Index, PathKeyItem::Index]),
                "[index].[index]",
            ),
        ];

        for (value, expected_value) in data {
            let value = value.to_string();
            assert_eq!(value.as_str(), *expected_value);
        }
    }

    /// Tests that keys are parsed correctly
    #[test]
    fn test_key_parsing() {
        let data = &[
            (test_path_key([PathKeyItem::Key("Key".into())]), "Key"),
            (test_path_key([PathKeyItem::Index]), "[index]"),
            (
                test_path_key([PathKeyItem::Index, PathKeyItem::Key("Nested".into())]),
                "[index].Nested",
            ),
            (
                test_path_key([
                    PathKeyItem::Key("Test".into()),
                    PathKeyItem::Index,
                    PathKeyItem::Key("Nested".into()),
                ]),
                "Test.[index].Nested",
            ),
            (
                test_path_key([
                    PathKeyItem::Key("Test".into()),
                    PathKeyItem::Index,
                    PathKeyItem::Key("Nested".into()),
                    PathKeyItem::Key("Deep".into()),
                ]),
                "Test.[index].Nested.Deep",
            ),
            (
                test_path_key([
                    PathKeyItem::Index,
                    PathKeyItem::Index,
                    PathKeyItem::Key("Deep".into()),
                ]),
                "[index].[index].Deep",
            ),
            (
                test_path_key([PathKeyItem::Index, PathKeyItem::Index]),
                "[index].[index]",
            ),
            // Empty strings are technically valid JSON keys
            (test_path_key([PathKeyItem::Key("".into())]), ""),
            (
                test_path_key([PathKeyItem::Key("".into()), PathKeyItem::Key("".into())]),
                ".",
            ),
            (
                test_path_key([PathKeyItem::Key("".into()), PathKeyItem::Index]),
                ".[index]",
            ),
        ];

        for (expected_value, value) in data {
            let value: PathKey = value.parse().unwrap();
            let value = Arc::new(value);
            assert_eq!(&value, expected_value);
        }
    }

    /// Tests that parsing errors are returned for invalid data
    #[test]
    fn test_key_parsing_err() {
        let data = &[
            ("Test\\", PathKeyParseError::DanglingEscape),
            ("Test\\a", PathKeyParseError::InvalidEscape('a')),
        ];

        for (value, expected_err) in data {
            let err: PathKeyParseError = value
                .parse::<PathKey>()
                .expect_err(&format!("{value} should produce {expected_err:?}"));
            assert_eq!(&err, expected_err);
        }
    }
}

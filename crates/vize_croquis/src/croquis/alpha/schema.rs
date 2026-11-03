//! Version pin for Croquis's structured production contracts.

use core::fmt;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// JSON contract version, independent of the opaque Levels facet grammar.
pub const ALPHA_CONTRACT_SCHEMA: u16 = 1;

/// A required schema marker that rejects every unsupported version on import.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AlphaSchema;

impl Serialize for AlphaSchema {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(ALPHA_CONTRACT_SCHEMA)
    }
}

impl<'de> Deserialize<'de> for AlphaSchema {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_u16(SchemaVisitor)
    }
}

struct SchemaVisitor;

impl<'de> Visitor<'de> for SchemaVisitor {
    type Value = AlphaSchema;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Croquis alpha contract schema {ALPHA_CONTRACT_SCHEMA}"
        )
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        if value == u64::from(ALPHA_CONTRACT_SCHEMA) {
            Ok(AlphaSchema)
        } else {
            Err(E::invalid_value(Unexpected::Unsigned(value), &self))
        }
    }
}

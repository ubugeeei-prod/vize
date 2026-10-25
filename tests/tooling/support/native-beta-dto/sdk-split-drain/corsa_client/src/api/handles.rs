use crate::{CorsaError, Result};
use corsa_core::fast::CompactString;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Unexpected, Visitor},
};
use std::fmt;

macro_rules! handle_type {
    ($name:ident) => {
        /// Opaque handle returned by Corsa.
        ///
        /// Handles are lightweight string wrappers and can be passed back to
        /// follow-up requests without parsing.
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub CompactString);

        impl $name {
            /// Returns the raw string representation of the handle.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(CompactString::from(value))
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(CompactString::from(value))
            }
        }
    };
}

macro_rules! numeric_wire_handle_type {
    ($name:ident) => {
        /// Opaque handle returned by Corsa.
        ///
        /// Handles are lightweight string wrappers and can be passed back to
        /// follow-up requests without parsing.
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub CompactString);

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserialize_string_or_number_handle(deserializer).map(Self)
            }
        }

        impl $name {
            /// Returns the raw string representation of the handle.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(CompactString::from(value))
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(CompactString::from(value))
            }
        }
    };
}

fn deserialize_string_or_number_handle<'de, D>(
    deserializer: D,
) -> std::result::Result<CompactString, D::Error>
where
    D: Deserializer<'de>,
{
    struct HandleVisitor;

    impl Visitor<'_> for HandleVisitor {
        type Value = CompactString;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or non-negative integer handle")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(CompactString::from(value))
        }

        fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(CompactString::from(value))
        }

        fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(CompactString::from(value.to_string()))
        }

        fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let value = u64::try_from(value)
                .map_err(|_| E::invalid_value(Unexpected::Signed(value), &self))?;
            self.visit_u64(value)
        }
    }

    deserializer.deserialize_any(HandleVisitor)
}

numeric_wire_handle_type!(SnapshotHandle);
handle_type!(ProjectHandle);
numeric_wire_handle_type!(SymbolHandle);
numeric_wire_handle_type!(TypeHandle);
numeric_wire_handle_type!(SignatureHandle);
handle_type!(NodeHandle);

/// Parsed representation of a [`NodeHandle`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedNodeHandle {
    /// Start offset in UTF-16 code units.
    pub pos: u32,
    /// End offset in UTF-16 code units.
    pub end: u32,
    /// TypeScript syntax kind numeric tag.
    pub kind: u16,
    /// Path component encoded into the handle.
    pub path: CompactString,
}

impl NodeHandle {
    /// Parses a node handle into offsets, syntax kind, and backing path.
    ///
    /// # Examples
    ///
    /// ```
    /// use corsa_client::NodeHandle;
    ///
    /// let parsed = NodeHandle::from("1.5.123./workspace/main.ts").parse()?;
    /// assert_eq!(parsed.pos, 1);
    /// assert_eq!(parsed.end, 5);
    /// assert_eq!(parsed.kind, 123);
    /// assert_eq!(parsed.path.as_str(), "/workspace/main.ts");
    /// # Ok::<(), corsa_client::CorsaError>(())
    /// ```
    ///
    /// Compact stable-runtime handles carry no offsets and are rejected here;
    /// use [`Self::declaring_path`] when only the declaring file is needed.
    pub fn parse(&self) -> Result<ParsedNodeHandle> {
        let mut parts = self.0.splitn(4, '.');
        let invalid = || CorsaError::InvalidHandle(self.0.clone());
        let pos = parts
            .next()
            .ok_or_else(invalid)?
            .parse::<u32>()
            .map_err(|_| invalid())?;
        let end = parts
            .next()
            .ok_or_else(invalid)?
            .parse::<u32>()
            .map_err(|_| invalid())?;
        let kind = parts
            .next()
            .ok_or_else(invalid)?
            .parse::<u16>()
            .map_err(|_| invalid())?;
        let path = parts.next().ok_or_else(invalid)?;
        if path.is_empty() || end < pos {
            return Err(invalid());
        }
        Ok(ParsedNodeHandle {
            pos,
            end,
            kind,
            path: path.into(),
        })
    }

    /// Returns the file path a node handle declares, in either wire format.
    ///
    /// Corsa emits two node handle shapes. Development runtimes use the full
    /// `<pos>.<end>.<kind>.<path>` form that [`Self::parse`] understands, while
    /// stable TypeScript 7 runtimes use a compact `<node id>.<kind>.<path>`
    /// form that carries no source offsets and so cannot be parsed into a
    /// range. Both still name the declaring file, which is enough for
    /// file-scoped work such as recovering parameter names from source.
    ///
    /// Returns `None` when the handle matches neither shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use corsa_client::NodeHandle;
    ///
    /// let full = NodeHandle::from("1.5.123./workspace/main.ts");
    /// assert_eq!(full.declaring_path().as_deref(), Some("/workspace/main.ts"));
    ///
    /// let compact = NodeHandle::from("42.176./workspace/main.ts");
    /// assert_eq!(compact.declaring_path().as_deref(), Some("/workspace/main.ts"));
    /// ```
    pub fn declaring_path(&self) -> Option<CompactString> {
        let parts = self.0.split('.').collect::<Vec<_>>();
        let [first, second, third, rest @ ..] = parts.as_slice() else {
            return None;
        };
        if first.parse::<u32>().is_err() || second.parse::<u32>().is_err() {
            return None;
        }
        // A numeric third field is the syntax kind of a full handle, so the
        // path starts after it. Otherwise the handle is the compact form and
        // the path starts at the third field itself.
        let path = if third.parse::<u32>().is_ok() {
            rest.join(".")
        } else {
            std::iter::once(third)
                .chain(rest)
                .copied()
                .collect::<Vec<_>>()
                .join(".")
        };
        (!path.is_empty()).then(|| CompactString::from(path))
    }
}

#[cfg(test)]
#[path = "handles_tests.rs"]
mod tests;

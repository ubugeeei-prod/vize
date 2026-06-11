//! Vue dialect selection (`vue.version`) config model.
//!
//! `vue.version` names the Vue language dialect a project is written in.
//! `"3"` — the default — is modern Vue 3; every other value selects a legacy
//! line whose toolchain support is opt-in (the `legacy` cargo feature in the
//! downstream crates). This module only owns parsing and validation of the
//! selector; resolving a dialect into parser/transform behavior happens once
//! per file in the legacy-capable crates (`vize_armature::legacy`).
//!
//! Parsing is strict on purpose: a version selector that silently fell back to
//! "some" line would mis-lint or mis-compile every file in the project, so
//! unknown or ambiguous values (such as `"0"`, which does not distinguish the
//! Vue 0.10 line from the 0.11-era line) are configuration errors with
//! actionable messages instead.

use serde::{Deserialize, Deserializer, de};

/// Vue language dialect selected by the `vue.version` config key.
///
/// Accepted config values are the bare version numbers `"3"` (default),
/// `"2.7"`, `"2"`, `"1"`, `"0.11"`, and `"0.10"`; a leading `v` (as printed in
/// diagnostics, e.g. `"v0.10"`) is also tolerated. Anything else fails config
/// parsing — see [`VueDialect::from_config_str`].
///
/// Vue 2.7 is kept distinct from Vue 2 because the lines differ on the script
/// side (2.7 backports `<script setup>` / composition APIs) even though they
/// share a template dialect downstream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VueDialect {
    /// Modern Vue 3 — the default dialect; not a legacy line.
    #[default]
    V3,
    /// Vue 2.7 (Vue 2 with the `<script setup>` / composition API backport).
    V2_7,
    /// Vue 2.x below 2.7.
    V2,
    /// Vue 1.x.
    V1,
    /// The Vue 0.11-era post-rewrite 0.x line.
    V0_11,
    /// Vue 0.10.x, the last pre-rewrite 0.x line (distinct from 0.11-era).
    V0_10,
}

impl VueDialect {
    /// Every selectable dialect, newest first.
    pub const ALL: [VueDialect; 6] = [
        Self::V3,
        Self::V2_7,
        Self::V2,
        Self::V1,
        Self::V0_11,
        Self::V0_10,
    ];

    /// The canonical `vue.version` config value for this dialect.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V3 => "3",
            Self::V2_7 => "2.7",
            Self::V2 => "2",
            Self::V1 => "1",
            Self::V0_11 => "0.11",
            Self::V0_10 => "0.10",
        }
    }

    /// Whether this dialect is a legacy (pre-Vue-3) line.
    pub const fn is_legacy(self) -> bool {
        !matches!(self, Self::V3)
    }

    /// Parse a `vue.version` config value.
    ///
    /// Unknown values are rejected rather than rounded to a nearby line, and
    /// `"0"` is rejected as ambiguous: the Vue 0.10 line and the 0.11-era
    /// line are distinct dialects (the 0.11.0 rewrite changed computed
    /// `$get`/`$set`, instantiation, and scope semantics), so the config must
    /// say which one it means.
    pub fn from_config_str(raw: &str) -> Result<Self, ParseVueDialectError> {
        let trimmed = raw.trim();
        let bare = trimmed.strip_prefix(['v', 'V']).unwrap_or(trimmed);
        match bare {
            "3" => Ok(Self::V3),
            "2.7" => Ok(Self::V2_7),
            "2" => Ok(Self::V2),
            "1" => Ok(Self::V1),
            "0.11" => Ok(Self::V0_11),
            "0.10" => Ok(Self::V0_10),
            "0" => Err(ParseVueDialectError {
                raw: raw.into(),
                kind: ParseVueDialectErrorKind::AmbiguousZero,
            }),
            _ => Err(ParseVueDialectError {
                raw: raw.into(),
                kind: ParseVueDialectErrorKind::Unknown,
            }),
        }
    }
}

/// Error produced when a `vue.version` value does not name a dialect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseVueDialectError {
    raw: crate::String,
    kind: ParseVueDialectErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseVueDialectErrorKind {
    /// `"0"` / `"v0"`: does not distinguish the 0.10 line from the 0.11-era
    /// line.
    AmbiguousZero,
    /// Any other unrecognized value.
    Unknown,
}

impl std::fmt::Display for ParseVueDialectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = self.raw.as_str();
        match self.kind {
            ParseVueDialectErrorKind::AmbiguousZero => write!(
                f,
                "ambiguous vue.version \"{raw}\": Vue 0.10 and the 0.11-era line are \
                 distinct dialects; use \"0.10\" or \"0.11\""
            ),
            ParseVueDialectErrorKind::Unknown => write!(
                f,
                "unknown vue.version \"{raw}\": expected \"3\" (default), \"2.7\", \
                 \"2\", \"1\", \"0.11\", or \"0.10\""
            ),
        }
    }
}

impl std::error::Error for ParseVueDialectError {}

impl<'de> Deserialize<'de> for VueDialect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VueDialectVisitor;

        impl de::Visitor<'_> for VueDialectVisitor {
            type Value = VueDialect;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Reached for non-string values, e.g. `"version": 2.7` in
                // JSON: steer the user toward the quoted string form.
                f.write_str(
                    "a Vue version string: \"3\", \"2.7\", \"2\", \"1\", \"0.11\", or \"0.10\" \
                     (quote the value; 0.10 and 0.11 are not representable as numbers)",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                VueDialect::from_config_str(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(VueDialectVisitor)
    }
}

/// Raw `vue` config section.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RawVueConfig {
    /// `vue.version` dialect selector; `None` when the key is absent.
    pub(crate) version: Option<VueDialect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_and_v_prefixed_values() {
        for dialect in VueDialect::ALL {
            assert_eq!(VueDialect::from_config_str(dialect.as_str()), Ok(dialect));
            let prefixed = crate::cstr!("v{}", dialect.as_str());
            assert_eq!(VueDialect::from_config_str(&prefixed), Ok(dialect));
        }
        assert_eq!(
            VueDialect::from_config_str(" 2.7 "),
            Ok(VueDialect::V2_7),
            "values are trimmed"
        );
    }

    #[test]
    fn only_vue3_is_not_legacy() {
        for dialect in VueDialect::ALL {
            assert_eq!(dialect.is_legacy(), dialect != VueDialect::V3);
        }
    }

    #[test]
    fn rejects_bare_zero_as_ambiguous() {
        for raw in ["0", "v0", "V0"] {
            let error = VueDialect::from_config_str(raw).unwrap_err();
            let message = crate::cstr!("{error}");
            assert!(message.contains("ambiguous"), "{message}");
            assert!(message.contains("\"0.10\""), "{message}");
            assert!(message.contains("\"0.11\""), "{message}");
        }
    }

    #[test]
    fn rejects_unknown_values_with_expected_list() {
        for raw in ["2.6", "4", "vue2", ""] {
            let error = VueDialect::from_config_str(raw).unwrap_err();
            let message = crate::cstr!("{error}");
            assert!(message.contains("unknown vue.version"), "{message}");
            assert!(message.contains("\"2.7\""), "{message}");
            assert!(message.contains("\"0.10\""), "{message}");
        }
    }

    #[test]
    fn deserializes_strings_and_rejects_numbers() {
        let dialect: VueDialect = serde_json::from_str("\"0.10\"").unwrap();
        assert_eq!(dialect, VueDialect::V0_10);

        let error = serde_json::from_str::<VueDialect>("2.7").unwrap_err();
        let message = crate::cstr!("{error}");
        assert!(message.contains("Vue version string"), "{message}");

        let error = serde_json::from_str::<VueDialect>("\"0\"").unwrap_err();
        let message = crate::cstr!("{error}");
        assert!(message.contains("ambiguous"), "{message}");
    }
}

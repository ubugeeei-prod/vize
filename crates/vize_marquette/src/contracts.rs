//! Versioned extension-contract surfaces (Davinci P6-8).
//!
//! A [`ContractSurface`] is the canonical, language-neutral description of
//! one version of a WIT contract package — its interfaces, types, functions
//! and worlds — together with the facts the handshake carries beside the
//! WIT: the integer protocol version, the serialized page schemas, and the
//! features each world requires. It gets the marquette treatment:
//!
//! - canonical serialization ([`canonical_surface_json`]) is deterministic
//!   (sorted maps, order-bearing lists kept in authored order) and
//!   fingerprinted ([`surface_fingerprint`]);
//! - [`compare_surfaces`] classifies every change as additive or breaking
//!   with the same [`CompatibilityChange`] records application contracts
//!   use;
//! - [`check_version_policy`] enforces how the package version and the
//!   protocol version must move for the classified change.
//!
//! The policy those functions implement is
//! `docs/davinci/contracts-compat-policy.md`. The optional `wit` feature
//! reads a surface from WIT sources with the Bytecode Alliance `wit-parser`
//! (`contracts::wit`).
//!
//! [`CompatibilityChange`]: crate::CompatibilityChange

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vize_s0::String;

mod compare;
mod version;
#[cfg(feature = "wit")]
pub mod wit;

pub use compare::{ContractSurfaceReport, compare_surfaces};
pub use version::{ContractVersion, VersionPolicyViolation, check_version_policy};

/// Serialized format name of a contract surface document.
pub const CONTRACT_SURFACE_FORMAT: &str = "vize.contract-surface";
/// Serialized format version of a contract surface document.
pub const CONTRACT_SURFACE_FORMAT_VERSION: u32 = 1;

/// One version of an extension-contract package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractSurface {
    /// Always [`CONTRACT_SURFACE_FORMAT`].
    pub format: String,
    /// Always [`CONTRACT_SURFACE_FORMAT_VERSION`].
    pub format_version: u32,
    /// The WIT package name, `namespace:name`.
    pub package: String,
    /// The WIT package's semver version, `MAJOR.MINOR.PATCH`.
    pub version: String,
    /// The integer protocol version the capability handshake negotiates.
    pub protocol_version: u32,
    /// Serialized payload schemas by page name (`s1-page` → `1`).
    pub pages: BTreeMap<String, u32>,
    /// Interfaces by name.
    pub interfaces: BTreeMap<String, InterfaceSurface>,
    /// Worlds by name.
    pub worlds: BTreeMap<String, WorldSurface>,
}

/// The named types and functions of one interface. Type references are WIT
/// spellings with named types qualified by their interface
/// (`types.diagnostic`, `list<types.diagnostic-part>`); `use` aliases are
/// transparent and never listed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceSurface {
    pub types: BTreeMap<String, TypeShape>,
    pub functions: BTreeMap<String, FunctionShape>,
}

/// The shape of one named type. Member order is part of the shape: the
/// canonical ABI lays records and cases out positionally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TypeShape {
    Record(Vec<Field>),
    Variant(Vec<Case>),
    Enum(Vec<String>),
    Flags(Vec<String>),
    Resource,
    /// Any other named type (`type bytes = list<u8>`), by its spelling.
    Alias(String),
}

/// A record field or function parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

/// A variant case and its optional payload type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub name: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
}

/// A function's signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionShape {
    pub params: Vec<Field>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// What a world imports from the host and exports to it, by interface name,
/// and the handshake features a guest of the world must offer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldSurface {
    pub imports: BTreeSet<String>,
    pub exports: BTreeSet<String>,
    pub required_features: BTreeSet<String>,
}

/// The canonical serialization: two-space pretty JSON, maps sorted by key,
/// one trailing newline. Equal surfaces serialize to equal bytes.
#[must_use]
pub fn canonical_surface_json(surface: &ContractSurface) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(surface).expect("a surface always serializes");
    bytes.push(b'\n');
    bytes
}

/// The lowercase SHA-256 of [`canonical_surface_json`].
#[must_use]
pub fn surface_fingerprint(surface: &ContractSurface) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(canonical_surface_json(surface));
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

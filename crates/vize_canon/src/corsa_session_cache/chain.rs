//! The `tsconfig-content` input: every config an `extends` chain reaches.

use core::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::String;
use vize_carton::hash::StableHasher128;

use crate::batch::virtual_project::{parse_jsonc_value, resolve_extended_tsconfig_path};

/// Tags that keep the fed byte stream unambiguous.
const READ: u8 = 1;
const MISSING: u8 = 0;
const UNRESOLVED: u8 = 2;
const CYCLE: u8 = 3;

/// The digest (32 hex digits) of the config at `tsconfig_path` and every
/// config its `extends` chain reaches — depth-first in TypeScript's order,
/// array entries in order — each fed as its path and exact bytes. A config
/// that cannot be read contributes a marker, and an `extends` that does not
/// resolve contributes its specifier, so creating or installing either later
/// changes the digest.
pub fn tsconfig_chain_digest(tsconfig_path: &Path) -> String {
    let mut hasher = StableHasher128::new();
    let mut seen = Vec::new();
    visit(tsconfig_path, &mut hasher, &mut seen);
    let mut hex = String::default();
    for byte in hasher.digest() {
        let _infallible = write!(hex, "{byte:02x}");
    }
    hex
}

fn visit(path: &Path, hasher: &mut StableHasher128, seen: &mut Vec<PathBuf>) {
    feed(hasher, path.to_string_lossy().as_bytes());
    if seen.iter().any(|visited| visited == path) {
        hasher.update(&[CYCLE]);
        return;
    }
    seen.push(path.to_path_buf());
    let Ok(bytes) = std::fs::read(path) else {
        hasher.update(&[MISSING]);
        return;
    };
    hasher.update(&[READ]);
    feed(hasher, &bytes);
    let Some(config) = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|text| parse_jsonc_value(text).ok())
    else {
        return;
    };
    for extends in extends_of(&config) {
        match resolve_extended_tsconfig_path(path, extends) {
            Some(target) => visit(&target, hasher, seen),
            None => {
                hasher.update(&[UNRESOLVED]);
                feed(hasher, extends.as_bytes());
            }
        }
    }
}

/// `extends` as a string or an array of strings (TypeScript 5.0+).
fn extends_of(config: &Value) -> Vec<&str> {
    match config.get("extends") {
        Some(Value::String(one)) => vec![one.as_str()],
        Some(Value::Array(many)) => many.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

/// Feed `bytes` length-prefixed, so adjacent fields cannot run together.
fn feed(hasher: &mut StableHasher128, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

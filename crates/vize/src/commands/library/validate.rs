//! Structural validation of `registry.json` mirroring
//! `npm/cli/schemas/vize-lib-registry.schema.json`.
//!
//! Field presence and `additionalProperties: false` are enforced by serde
//! (`deny_unknown_fields`); this module checks every value constraint the
//! schema expresses plus the cross-item invariants it documents, so a
//! third-party registry is held to exactly the contract the first-party
//! packages are built against.

use std::path::Path;

use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{content_hash, validate_relative_path};
use super::registry::{RegistryItem, RegistryManifest, SUPPORTED_SCHEMA_VERSION};

const ITEM_KINDS: [&str; 2] = ["ui", "composable"];
const FILE_ROLES: [&str; 5] = ["entry", "component", "style", "types", "module"];
const NPM_KINDS: [&str; 2] = ["peer", "runtime"];

/// `^[a-z0-9][a-z0-9-]*$`
pub fn is_item_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

/// `^[0-9a-f]{64}$`
pub fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}

fn fail(path: &Path, message: impl std::fmt::Display) -> LibError {
    LibError::new(cstr!("invalid registry {}: {message}", path.display()))
}

fn validate_item(path: &Path, item: &RegistryItem, names: &[&str]) -> LibResult<()> {
    let at = cstr!("item {:?}", item.name.as_str());
    if !is_item_name(&item.name) {
        return Err(fail(
            path,
            cstr!("{at}: name must match ^[a-z0-9][a-z0-9-]*$"),
        ));
    }
    if !ITEM_KINDS.contains(&item.kind.as_str()) {
        return Err(fail(path, cstr!("{at}: kind must be ui or composable")));
    }
    if item.title.is_empty() || item.description.is_empty() {
        return Err(fail(
            path,
            cstr!("{at}: title and description must be non-empty"),
        ));
    }
    if item.aliases.iter().any(|alias| alias.is_empty()) {
        return Err(fail(path, cstr!("{at}: aliases must be non-empty")));
    }
    if !(item.package_subpath == "." || item.package_subpath.starts_with("./")) {
        return Err(fail(path, cstr!("{at}: packageSubpath must start with ./")));
    }
    if item.files.is_empty() {
        return Err(fail(path, cstr!("{at}: files must not be empty")));
    }
    validate_relative_path(&item.entry)?;
    let mut seen: Vec<&str> = Vec::with_capacity(item.files.len());
    for file in &item.files {
        validate_relative_path(&file.path)?;
        if !FILE_ROLES.contains(&file.role.as_str()) {
            return Err(fail(
                path,
                cstr!("{at}: unknown file role {:?}", file.role.as_str()),
            ));
        }
        if !is_sha256(&file.sha256) {
            return Err(fail(
                path,
                cstr!("{at}: {} has a malformed sha256", file.path),
            ));
        }
        if seen.contains(&file.path.as_str()) {
            return Err(fail(path, cstr!("{at}: duplicate file {}", file.path)));
        }
        seen.push(&file.path);
    }
    if !seen.contains(&item.entry.as_str()) {
        return Err(fail(
            path,
            cstr!("{at}: entry {} is not one of its files", item.entry),
        ));
    }
    for dependency in &item.dependencies {
        if dependency.name.is_empty()
            || dependency.range.is_empty()
            || !NPM_KINDS.contains(&dependency.kind.as_str())
        {
            return Err(fail(
                path,
                cstr!(
                    "{at}: malformed npm dependency {:?}",
                    dependency.name.as_str()
                ),
            ));
        }
    }
    for dependency in &item.registry_dependencies {
        if !names.contains(&dependency.as_str()) || *dependency == item.name {
            return Err(fail(
                path,
                cstr!("{at}: unknown registry dependency {dependency}"),
            ));
        }
    }
    if !is_sha256(&item.content_hash)
        || item.content_hash
            != content_hash(
                item.files
                    .iter()
                    .map(|file| (file.path.as_str(), file.sha256.as_str())),
            )
    {
        return Err(fail(path, cstr!("{at}: inconsistent contentHash")));
    }
    Ok(())
}

/// Validate a parsed manifest; `path` is used in messages only.
pub fn validate_manifest(path: &Path, manifest: &RegistryManifest) -> LibResult<()> {
    if manifest.registry_kind != "vize-lib" {
        return Err(fail(path, "registryKind must be \"vize-lib\""));
    }
    if manifest.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(LibError::new(cstr!(
            "{} uses registry schema {}; this vize understands {} (upgrade vize)",
            path.display(),
            manifest.schema_version,
            SUPPORTED_SCHEMA_VERSION
        )));
    }
    if manifest.package.name.is_empty() || manifest.package.version.is_empty() {
        return Err(fail(path, "package name and version must be non-empty"));
    }
    if !ITEM_KINDS.contains(&manifest.kind.as_str()) {
        return Err(fail(path, "kind must be ui or composable"));
    }
    if manifest.files_directory != "files" {
        return Err(fail(path, "filesDirectory must be \"files\""));
    }
    validate_relative_path(&manifest.default_target_directory)?;
    let names: Vec<&str> = manifest
        .items
        .iter()
        .map(|item| item.name.as_str())
        .collect();
    for (index, name) in names.iter().enumerate() {
        if names.iter().skip(index + 1).any(|other| other == name) {
            return Err(fail(path, cstr!("duplicate item {name}")));
        }
    }
    let mut owners: Vec<(&str, &str)> = Vec::new();
    for item in &manifest.items {
        validate_item(path, item, &names)?;
        for file in &item.files {
            if let Some((_, owner)) = owners.iter().find(|(seen, _)| *seen == file.path.as_str()) {
                return Err(fail(
                    path,
                    cstr!(
                        "{} is published by both {owner} and {}",
                        file.path,
                        item.name
                    ),
                ));
            }
            owners.push((&file.path, &item.name));
        }
    }
    // registryDependencies must be a transitive closure (the schema contract).
    for item in &manifest.items {
        for dependency in &item.registry_dependencies {
            let nested = manifest
                .items
                .iter()
                .find(|candidate| candidate.name == *dependency)
                .map(|candidate| candidate.registry_dependencies.as_slice())
                .unwrap_or_default();
            if let Some(missing) = nested
                .iter()
                .find(|name| **name != item.name && !item.registry_dependencies.contains(name))
            {
                let message: String = cstr!(
                    "item {}: registryDependencies misses {missing} (via {dependency})",
                    item.name
                );
                return Err(fail(path, message));
            }
        }
    }
    Ok(())
}

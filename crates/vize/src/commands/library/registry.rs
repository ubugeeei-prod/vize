//! Published source registry (`registry/registry.json`) model and queries.
//!
//! The shape is documented by `npm/cli/schemas/vize-lib-registry.schema.json`
//! and produced by `npm/ui/scripts/source-registry-bundle`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{join_relative, sha256_hex};
use super::validate::validate_manifest;

/// Registry document versions this CLI understands.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Root of `registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryManifest {
    pub schema_version: u32,
    pub registry_kind: String,
    pub package: RegistryPackage,
    pub kind: String,
    pub files_directory: String,
    pub default_target_directory: String,
    pub items: Vec<RegistryItem>,
}

/// Package and version that published a registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryPackage {
    pub name: String,
    pub version: String,
}

/// One pullable item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryItem {
    pub name: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub package_subpath: String,
    pub entry: String,
    pub files: Vec<RegistryFile>,
    pub registry_dependencies: Vec<String>,
    pub dependencies: Vec<NpmDependency>,
    pub content_hash: String,
}

/// One published file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryFile {
    pub path: String,
    pub role: String,
    pub sha256: String,
    pub size: u64,
}

/// One npm package an item imports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpmDependency {
    pub name: String,
    pub range: String,
    pub kind: String,
}

/// A registry loaded from disk.
#[derive(Debug)]
pub struct LoadedRegistry {
    pub manifest: RegistryManifest,
    /// Path of `registry.json`.
    pub manifest_path: PathBuf,
    /// How the registry was located (for messages).
    pub origin: String,
    /// Keeps a temporary `npm pack` / download directory alive.
    pub _temporary: Option<tempfile::TempDir>,
    /// For URL registries: base URL and downloader for files fetched on demand.
    pub remote: Option<RemoteFiles>,
}

/// Lazily downloaded files of an `https://` registry.
#[derive(Debug, Clone)]
pub struct RemoteFiles {
    /// URL of the directory that holds `registry.json` (no trailing slash).
    pub base_url: String,
    /// curl executable.
    pub curl: std::ffi::OsString,
}

impl LoadedRegistry {
    /// Read and validate `registry.json`.
    pub fn load(manifest_path: &Path, origin: impl Into<String>) -> LibResult<Self> {
        let bytes =
            fs::read(manifest_path).map_err(|error| LibError::io("read", manifest_path, &error))?;
        let manifest: RegistryManifest = serde_json::from_slice(&bytes).map_err(|error| {
            LibError::new(cstr!(
                "invalid registry {}: {error}",
                manifest_path.display()
            ))
        })?;
        validate_manifest(manifest_path, &manifest)?;
        Ok(Self {
            manifest,
            manifest_path: manifest_path.to_path_buf(),
            origin: origin.into(),
            _temporary: None,
            remote: None,
        })
    }

    /// `name@version` of the publishing package.
    pub fn package_label(&self) -> String {
        cstr!(
            "{}@{}",
            self.manifest.package.name,
            self.manifest.package.version
        )
    }

    /// Read a published file and verify it against the manifest digest.
    pub fn read_file(&self, file: &RegistryFile) -> LibResult<Vec<u8>> {
        let root = self
            .manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let path = join_relative(
            &join_relative(root, &self.manifest.files_directory)?,
            &file.path,
        )?;
        if let Some(remote) = &self.remote
            && !path.is_file()
        {
            let url = cstr!(
                "{}/{}/{}",
                remote.base_url,
                self.manifest.files_directory,
                file.path
            );
            super::fetch::download(&remote.curl, &url, &path)?;
        }
        let bytes = fs::read(&path).map_err(|error| LibError::io("read", &path, &error))?;
        let actual = sha256_hex(&bytes);
        if actual != file.sha256 {
            return Err(LibError::new(cstr!(
                "{} does not match its registry digest (expected {}, found {actual})",
                path.display(),
                file.sha256
            )));
        }
        Ok(bytes)
    }

    /// Find an item by canonical name, then exact alias, then case-insensitive alias.
    pub fn find(&self, query: &str) -> Option<&RegistryItem> {
        let items = &self.manifest.items;
        items
            .iter()
            .find(|item| item.name == query)
            .or_else(|| {
                items
                    .iter()
                    .find(|item| item.aliases.iter().any(|alias| alias == query))
            })
            .or_else(|| {
                let lowered = query.to_lowercase();
                items.iter().find(|item| {
                    item.name.to_lowercase() == lowered
                        || item
                            .aliases
                            .iter()
                            .any(|alias| alias.to_lowercase() == lowered)
                })
            })
    }

    /// Item by exact canonical name.
    pub fn item(&self, name: &str) -> Option<&RegistryItem> {
        self.manifest.items.iter().find(|item| item.name == name)
    }

    /// The item followed by its registry dependencies, each exactly once.
    pub fn closure(&self, item: &RegistryItem) -> LibResult<Vec<&RegistryItem>> {
        let mut resolved = Vec::with_capacity(item.registry_dependencies.len() + 1);
        let Some(root) = self.item(&item.name) else {
            return Err(LibError::new(cstr!("unknown item {}", item.name)));
        };
        resolved.push(root);
        for name in &item.registry_dependencies {
            let dependency = self.item(name).ok_or_else(|| {
                LibError::new(cstr!(
                    "{} depends on {name}, which {} does not publish",
                    item.name,
                    self.package_label()
                ))
            })?;
            if !resolved.iter().any(|seen| seen.name == dependency.name) {
                resolved.push(dependency);
            }
        }
        Ok(resolved)
    }

    /// Items whose name, title, description, or aliases contain every query token.
    pub fn search(&self, query: &str) -> Vec<&RegistryItem> {
        let tokens: Vec<String> = query
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(|token| String::from(token.to_lowercase()))
            .collect();
        if tokens.is_empty() {
            return Vec::new();
        }
        self.manifest
            .items
            .iter()
            .filter(|item| {
                let mut haystack = item.name.to_lowercase();
                for value in [&item.title, &item.description]
                    .into_iter()
                    .chain(item.aliases.iter())
                {
                    haystack.push(' ');
                    haystack.push_str(&value.to_lowercase());
                }
                tokens.iter().all(|token| haystack.contains(token.as_str()))
            })
            .collect()
    }
}

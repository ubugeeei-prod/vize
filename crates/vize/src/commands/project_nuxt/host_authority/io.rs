//! Counted, bounded IO owned by one Nuxt scope attempt. No global witness cache.

use std::fs::{self, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_l0::{FxHashMap, String, profiler::global_profiler};

pub(crate) const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_DIRECTORY_ENTRIES: usize = 4096;
const MAX_MANIFEST_READS: u64 = 6;

#[derive(Debug, Clone)]
pub(crate) struct ManifestEvidence {
    pub(crate) package_root: PathBuf,
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) module: bool,
    pub(crate) exports: Option<Value>,
    pub(crate) has_imports: bool,
    pub(crate) imports: Option<Value>,
    pub(crate) manager_overrides: bool,
    pub(crate) inlined_dependencies: FxHashMap<String, String>,
    pub(crate) bytes: u64,
}

#[derive(Clone)]
pub(crate) enum PathMetadata {
    Missing,
    Present(Metadata),
    Unknown,
}

#[derive(Clone)]
pub(crate) enum PackageDirectory {
    Missing,
    Canonical(PathBuf),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    File,
    Directory,
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) name: std::ffi::OsString,
    pub(crate) kind: EntryKind,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct IoCounts {
    pub(crate) metadata: u64,
    pub(crate) canonicalize: u64,
    pub(crate) package_lookups: u64,
    pub(crate) manifest_opens: u64,
    pub(crate) manifest_reads: u64,
    pub(crate) manifest_bytes: u64,
    pub(crate) manifest_parses: u64,
    pub(crate) read_dir: u64,
    pub(crate) entries: u64,
    pub(crate) file_types: u64,
}

#[derive(Default)]
pub(crate) struct ProbeIo {
    pub(crate) counts: IoCounts,
    metadata: FxHashMap<PathBuf, PathMetadata>,
    canonical: FxHashMap<PathBuf, Option<PathBuf>>,
    manifests: FxHashMap<PathBuf, Option<ManifestEvidence>>,
    directories: FxHashMap<PathBuf, Option<Vec<Entry>>>,
}

impl ProbeIo {
    pub(crate) fn metadata(&mut self, path: &Path) -> PathMetadata {
        if let Some(cached) = self.metadata.get(path) {
            return cached.clone();
        }
        self.counts.metadata += 1;
        let result = match fs::symlink_metadata(path) {
            Ok(metadata) => PathMetadata::Present(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => PathMetadata::Missing,
            Err(_) => PathMetadata::Unknown,
        };
        self.metadata.insert(path.to_path_buf(), result.clone());
        result
    }

    pub(crate) fn directory(&mut self, path: &Path) -> Option<Vec<Entry>> {
        if let Some(cached) = self.directories.get(path) {
            return cached.clone();
        }
        if self.counts.read_dir >= 4 {
            return None;
        }
        self.counts.read_dir += 1;
        let result = (|| {
            let mut result = Vec::new();
            for entry in fs::read_dir(path).ok()? {
                let entry = entry.ok()?;
                self.counts.entries += 1;
                if result.len() == MAX_DIRECTORY_ENTRIES {
                    return None;
                }
                self.counts.file_types += 1;
                let kind = entry.file_type().ok()?;
                result.push(Entry {
                    name: entry.file_name(),
                    kind: if kind.is_symlink() {
                        EntryKind::Unknown
                    } else if kind.is_dir() {
                        EntryKind::Directory
                    } else if kind.is_file() {
                        EntryKind::File
                    } else {
                        EntryKind::Unknown
                    },
                });
            }
            Some(result)
        })();
        self.directories.insert(path.to_path_buf(), result.clone());
        result
    }

    pub(crate) fn canonical_directory(
        &mut self,
        path: &Path,
        package_link: bool,
    ) -> Option<PathBuf> {
        match self.metadata(path) {
            PathMetadata::Present(metadata)
                if metadata.is_dir() || (package_link && metadata.file_type().is_symlink()) => {}
            _ => return None,
        }
        if let Some(cached) = self.canonical.get(path) {
            return cached.clone();
        }
        self.counts.canonicalize += 1;
        let result = fs::canonicalize(path).ok().and_then(|physical| {
            if physical.components().count() > 64 {
                return None;
            }
            match self.metadata(&physical) {
                PathMetadata::Present(metadata)
                    if metadata.is_dir() && !metadata.file_type().is_symlink() =>
                {
                    Some(physical)
                }
                _ => None,
            }
        });
        self.canonical.insert(path.to_path_buf(), result.clone());
        result
    }

    pub(crate) fn package_directory(&mut self, path: &Path) -> PackageDirectory {
        match self.metadata(path) {
            PathMetadata::Missing => PackageDirectory::Missing,
            PathMetadata::Present(_) => self
                .canonical_directory(path, true)
                .map_or(PackageDirectory::Unknown, PackageDirectory::Canonical),
            PathMetadata::Unknown => PackageDirectory::Unknown,
        }
    }

    pub(crate) fn regular_file(&mut self, path: &Path) -> bool {
        matches!(self.metadata(path), PathMetadata::Present(metadata)
            if metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0)
    }

    pub(crate) fn package_lookup_request(&mut self) {
        self.counts.package_lookups += 1;
    }

    pub(crate) fn manifest(&mut self, root: &Path) -> Option<ManifestEvidence> {
        let path = root.join("package.json");
        if let Some(cached) = self.manifests.get(&path) {
            return cached.clone();
        }
        let result = self.read_manifest(root, &path);
        self.manifests.insert(path, result.clone());
        result
    }

    fn read_manifest(&mut self, root: &Path, path: &Path) -> Option<ManifestEvidence> {
        if self.counts.manifest_opens >= MAX_MANIFEST_READS {
            return None;
        }
        match self.metadata(path) {
            PathMetadata::Present(metadata)
                if metadata.is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.len() <= MAX_MANIFEST_BYTES => {}
            _ => return None,
        }
        self.counts.manifest_opens += 1;
        let mut file = fs::File::open(path).ok()?;
        self.counts.metadata += 1;
        let before = file.metadata().ok()?;
        if !before.is_file() || before.len() > MAX_MANIFEST_BYTES {
            return None;
        }
        let mut bytes = Vec::new();
        self.counts.manifest_reads += 1;
        let read = (&mut file).take(MAX_MANIFEST_BYTES).read_to_end(&mut bytes);
        self.counts.manifest_bytes += bytes.len() as u64;
        read.ok()?;
        self.counts.metadata += 1;
        let after = file.metadata().ok()?;
        if bytes.len() as u64 > MAX_MANIFEST_BYTES
            || before.len() != after.len()
            || before.modified().ok()? != after.modified().ok()?
            || bytes.len() as u64 != after.len()
        {
            return None;
        }
        self.counts.manifest_parses += 1;
        let value: Value = serde_json::from_slice(&bytes).ok()?;
        let object = value.as_object()?;
        for key in ["name", "version"] {
            if object.get(key).is_some_and(|value| !value.is_string()) {
                return None;
            }
        }
        let mut inlined_dependencies = FxHashMap::default();
        if let Some(inlined) = object.get("inlinedDependencies") {
            for (name, version) in inlined.as_object()? {
                inlined_dependencies.insert(name.as_str().into(), version.as_str()?.into());
            }
        }
        Some(ManifestEvidence {
            package_root: root.to_path_buf(),
            name: object.get("name").and_then(Value::as_str).map(Into::into),
            version: object
                .get("version")
                .and_then(Value::as_str)
                .map(Into::into),
            module: object.get("type").and_then(Value::as_str) == Some("module"),
            exports: object.get("exports").cloned(),
            has_imports: object.contains_key("imports"),
            imports: object.get("imports").cloned(),
            manager_overrides: ["pnpm", "overrides", "resolutions"]
                .iter()
                .any(|key| object.contains_key(*key)),
            inlined_dependencies,
            bytes: bytes.len() as u64,
        })
    }
}

impl Drop for ProbeIo {
    fn drop(&mut self) {
        let profiler = global_profiler();
        profiler.record_counter("cli.lint.nuxt_scope.invocations", 1);
        for (name, value) in [
            (
                "cli.lint.nuxt_scope.metadata_requests",
                self.counts.metadata,
            ),
            (
                "cli.lint.nuxt_scope.canonicalize_requests",
                self.counts.canonicalize,
            ),
            (
                "cli.lint.nuxt_scope.package_lookup_requests",
                self.counts.package_lookups,
            ),
            (
                "cli.lint.nuxt_scope.manifest_open_requests",
                self.counts.manifest_opens,
            ),
            (
                "cli.lint.nuxt_scope.manifest_read_requests",
                self.counts.manifest_reads,
            ),
            (
                "cli.lint.nuxt_scope.manifest_bytes",
                self.counts.manifest_bytes,
            ),
            (
                "cli.lint.nuxt_scope.manifest_json_parse_requests",
                self.counts.manifest_parses,
            ),
            (
                "cli.lint.nuxt_scope.read_dir_requests",
                self.counts.read_dir,
            ),
            ("cli.lint.nuxt_scope.directory_entries", self.counts.entries),
            (
                "cli.lint.nuxt_scope.file_type_requests",
                self.counts.file_types,
            ),
        ] {
            profiler.record_counter(name, value);
        }
    }
}

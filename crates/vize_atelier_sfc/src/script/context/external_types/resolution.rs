use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock};

use vize_carton::{FxHashMap, String, ToCompactString};

use super::super::batch_epoch::{NO_EPOCH, current_batch_epoch};

#[path = "exports_patterns.rs"]
mod exports_patterns;

use self::exports_patterns::{ExportsTypes, exports_types_entry};

const RESOLVE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".js", ".jsx", ".vue",
];
const INDEX_CANDIDATES: &[&str] = &[
    "index.ts",
    "index.tsx",
    "index.d.ts",
    "index.mts",
    "index.cts",
    "index.js",
    "index.jsx",
    "index.vue",
];

/// A resolved path plus the batch epoch in which its existence was last
/// confirmed. A read hit can stamp the epoch forward under the shared guard.
struct CachedPath {
    path: PathBuf,
    validated_epoch: AtomicU64,
}

/// Decide whether a cached resolved path is still usable, paying the `is_file`
/// `stat` only when the entry has not already been confirmed this batch.
fn cached_path_is_fresh(entry: &CachedPath, epoch: u64) -> bool {
    if epoch != NO_EPOCH && entry.validated_epoch.load(Ordering::Relaxed) == epoch {
        return true;
    }
    if entry.path.is_file() {
        entry.validated_epoch.store(epoch, Ordering::Relaxed);
        true
    } else {
        false
    }
}

/// Base-file canonicalization cache: `(cwd, filename) -> canonical path`.
/// Outside a batch a hit is revalidated with one `is_file` check; within a
/// batch the first hit revalidates and later hits reuse it. Failures are never
/// cached, so files created later are picked up.
static BASE_CANON_CACHE: LazyLock<RwLock<FxHashMap<(PathBuf, String), CachedPath>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

/// Canonicalize the compiled file's own path, falling back to the original
/// path for virtual filenames that do not exist on disk.
pub(super) fn canonical_base_file(filename: &str) -> PathBuf {
    let path = PathBuf::from(filename);
    let cwd = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    let key = (cwd, filename.to_compact_string());
    let epoch = current_batch_epoch();
    if let Ok(cache) = BASE_CANON_CACHE.read()
        && let Some(entry) = cache.get(&key)
        && cached_path_is_fresh(entry, epoch)
    {
        return entry.path.clone();
    }

    match path.canonicalize() {
        Ok(canonical) => {
            if let Ok(mut cache) = BASE_CANON_CACHE.write() {
                cache.insert(
                    key,
                    CachedPath {
                        path: canonical.clone(),
                        validated_epoch: AtomicU64::new(epoch),
                    },
                );
            }
            canonical
        }
        Err(_) => path,
    }
}

/// Positive resolution cache: `(importing dir, specifier) -> resolved path`.
/// Misses are not cached so newly created files remain discoverable.
static RESOLVE_CACHE: LazyLock<RwLock<FxHashMap<(PathBuf, String), CachedPath>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));

pub(super) fn resolve_import_path(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    let dir = current_file.parent()?.to_path_buf();
    let key = (dir, specifier.to_compact_string());
    let epoch = current_batch_epoch();
    if let Ok(cache) = RESOLVE_CACHE.read()
        && let Some(entry) = cache.get(&key)
        && cached_path_is_fresh(entry, epoch)
    {
        return Some(entry.path.clone());
    }

    let resolved = resolve_import_path_uncached(current_file, specifier)?;
    if let Ok(mut cache) = RESOLVE_CACHE.write() {
        cache.insert(
            key,
            CachedPath {
                path: resolved.clone(),
                validated_epoch: AtomicU64::new(epoch),
            },
        );
    }
    Some(resolved)
}

fn resolve_import_path_uncached(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if let Some(alias_path) = resolve_at_src_alias(current_file, specifier) {
        return Some(alias_path);
    }

    if !specifier.starts_with('.') && !specifier.starts_with('/') {
        return resolve_bare_specifier(current_file, specifier);
    }

    let base_dir = current_file.parent()?;
    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        base_dir.join(specifier)
    };

    resolve_candidate_path(candidate)
}

/// Resolve a bare specifier to package type declarations through
/// `node_modules`. Files already inside `node_modules` follow only relative
/// imports, avoiding unrelated dependency type graphs.
fn resolve_bare_specifier(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    if specifier.starts_with('#') || specifier.starts_with("node:") {
        return None;
    }
    if current_file
        .components()
        .any(|component| component.as_os_str() == "node_modules")
    {
        return None;
    }

    let (package, subpath) = split_package_specifier(specifier)?;
    for dir in current_file.ancestors().skip(1) {
        let package_dir = dir.join("node_modules").join(&package);
        if package_dir.is_dir() {
            return resolve_package_types(&package_dir, &subpath);
        }
    }
    None
}

/// Split `@scope/name/sub/path` / `name/sub/path` into package name and
/// subpath.
fn split_package_specifier(specifier: &str) -> Option<(String, String)> {
    let segment_count = if specifier.starts_with('@') { 2 } else { 1 };
    let mut split_at = 0;
    let mut seen = 0;
    for (index, byte) in specifier.bytes().enumerate() {
        if byte == b'/' {
            seen += 1;
            if seen == segment_count {
                split_at = index;
                break;
            }
        }
    }
    if seen < segment_count {
        split_at = specifier.len();
    }
    let package = &specifier[..split_at];
    if package.is_empty() {
        return None;
    }
    let subpath = specifier[split_at..].trim_start_matches('/');
    Some((package.to_compact_string(), subpath.to_compact_string()))
}

fn resolve_package_types(package_dir: &Path, subpath: &str) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok();
    let manifest: Option<serde_json::Value> =
        manifest.and_then(|raw| serde_json::from_str(&raw).ok());

    if subpath.is_empty() {
        if let Some(manifest) = &manifest {
            // Modern resolution honours `exports` ahead of the legacy
            // top-level `types`/`typings` entry points.
            if let Some(types) = exports_root_types_entry(manifest)
                && let Some(path) = resolve_candidate_path(package_dir.join(types))
            {
                return Some(path);
            }
            if let Some(types) = manifest
                .get("types")
                .or_else(|| manifest.get("typings"))
                .and_then(|value| value.as_str())
                && let Some(path) = resolve_candidate_path(package_dir.join(types))
            {
                return Some(path);
            }

            // Workspace `types` files may exist only after a build. Fall back
            // to checked-in TS entries so inherited prop keys remain visible.
            for field in ["module", "main"] {
                if let Some(entry) = manifest.get(field).and_then(|value| value.as_str())
                    && let Some(path) = resolve_candidate_path(package_dir.join(entry))
                {
                    return Some(path);
                }
            }
        }
        return resolve_candidate_path(package_dir.join("index.d.ts"))
            .or_else(|| resolve_candidate_path(package_dir.join("index")));
    }

    if let Some(manifest) = &manifest {
        let mut export_key = String::from("./");
        export_key.push_str(subpath);
        match exports_types_entry(manifest, export_key.as_str()) {
            // A `null` target blocks the subpath, so the physical declaration
            // beside it must stay unreachable.
            Some(ExportsTypes::Excluded) => return None,
            Some(ExportsTypes::Types(types)) => {
                if let Some(path) = resolve_candidate_path(package_dir.join(types)) {
                    return Some(path);
                }
            }
            None => {}
        }
    }
    resolve_candidate_path(package_dir.join(subpath))
}

/// Find the `types` condition for the package root. The root entry is either
/// the explicit `"."` subpath or, when no subpath keys are present, a condition
/// map placed directly under `exports`.
fn exports_root_types_entry(manifest: &serde_json::Value) -> Option<String> {
    let exports = manifest.get("exports")?;
    if let Some(root) = exports.get(".") {
        return find_types_condition(root);
    }
    let conditions = exports.as_object()?;
    if conditions.keys().any(|key| key.starts_with('.')) {
        return None;
    }
    find_types_condition(exports)
}

/// Read a `types` condition out of an `exports` value; conditions may nest.
fn find_types_condition(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(types) = map.get("types").and_then(|value| value.as_str()) {
                return Some(types.to_compact_string());
            }
            map.values().find_map(find_types_condition)
        }
        _ => None,
    }
}

pub(super) fn resolve_at_src_alias(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier.strip_prefix("@/")?;
    let src_dir = current_file
        .parent()?
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "src"))?;

    resolve_candidate_path(src_dir.join(rest))
}

fn resolve_candidate_path(candidate: PathBuf) -> Option<PathBuf> {
    // This resolver only serves type-bearing imports. For JS-shaped
    // specifiers, prefer the TypeScript/declaration counterpart even when the
    // published runtime file exists beside it.
    if let Some(ts_source_path) = resolve_ts_source_path_for_js_specifier(&candidate) {
        return Some(ts_source_path);
    }

    if candidate.is_file() {
        return canonicalize_or_original(candidate);
    }

    for ext in RESOLVE_EXTENSIONS {
        let mut with_ext = candidate.clone().into_os_string();
        with_ext.push(ext);
        let path = PathBuf::from(with_ext);
        if path.is_file() {
            return canonicalize_or_original(path);
        }
    }

    if candidate.is_dir() {
        for index_name in INDEX_CANDIDATES {
            let path = candidate.join(index_name);
            if path.is_file() {
                return canonicalize_or_original(path);
            }
        }
    }

    None
}

fn resolve_ts_source_path_for_js_specifier(candidate: &Path) -> Option<PathBuf> {
    let extension = candidate.extension()?.to_str()?;
    let source_extensions: &[&str] = match extension {
        "js" => &["ts", "tsx", "d.ts"],
        "jsx" => &["tsx", "ts", "d.ts"],
        "mjs" => &["mts", "d.mts", "ts", "d.ts"],
        "cjs" => &["cts", "d.cts", "ts", "d.ts"],
        _ => return None,
    };

    for source_extension in source_extensions {
        let source_candidate = candidate.with_extension(source_extension);
        if source_candidate.is_file() {
            return canonicalize_or_original(source_candidate);
        }
    }

    None
}

fn canonicalize_or_original(path: PathBuf) -> Option<PathBuf> {
    match path.canonicalize() {
        Ok(canonical) => Some(canonical),
        Err(_) if path.exists() => Some(path),
        Err(_) => None,
    }
}

pub(super) fn path_key(path: &Path) -> String {
    path.to_string_lossy().as_ref().to_compact_string()
}

#[cfg(test)]
#[path = "resolution_tests.rs"]
mod tests;

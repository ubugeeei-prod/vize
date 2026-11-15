//! TypeScript-compatible local file probing for check import discovery.

use std::path::{Path, PathBuf};

use vize_l0::{FxHashMap, String, cstr};

use super::super::path_cache::CanonicalPathCache;
use super::ImportFileOptions;

pub(super) fn absolutize(
    path: &Path,
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
) -> Option<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    Some(canonical_paths.canonicalize(&joined))
}

/// Positive local import edges resolved during one check invocation.
///
/// The look-ahead frontier, the ordered authored walk and overlapping
/// registration closures traverse the same relative edges. A successful
/// resolution is stable enough to share between those traversals, while a
/// negative answer deliberately remains uncached so a file created between
/// collections is still discovered. Cached targets are revalidated before
/// reuse so deletion or extension replacement follows the original probes.
#[derive(Default)]
pub(super) struct LocalImportResolutionCache {
    resolved: FxHashMap<(ImportFileOptions, PathBuf), PathBuf>,
    #[cfg(test)]
    hits: usize,
    #[cfg(test)]
    probes: usize,
}

impl LocalImportResolutionCache {
    pub(super) fn resolve(
        &mut self,
        base: &Path,
        canonical_paths: &mut CanonicalPathCache,
        options: ImportFileOptions,
    ) -> Option<PathBuf> {
        let key = (options, base.to_path_buf());
        if let Some(target) = self.resolved.get(&key)
            && target.is_file()
        {
            #[cfg(test)]
            {
                self.hits += 1;
            }
            return Some(target.clone());
        }
        // A stale positive may otherwise hide a replacement with another
        // extension. Negative answers are never inserted in the first place.
        self.resolved.remove(&key);
        #[cfg(test)]
        {
            self.probes += 1;
        }
        let resolved = resolve_import_base(base, canonical_paths, options);
        if let Some(target) = resolved.as_ref() {
            self.resolved.insert(key, target.clone());
        }
        resolved
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> (usize, usize, usize) {
        (self.resolved.len(), self.hits, self.probes)
    }
}

/// Resolution on the authored import walk is the hot path, so consulted
/// candidates are only materialized when a caller needs them for invalidation.
pub(crate) fn resolve_import_base(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> Option<PathBuf> {
    probe_import_base(base, canonical_paths, options, None)
}

pub(crate) fn resolve_import_base_with_inputs(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> (Option<PathBuf>, Vec<PathBuf>) {
    let mut inputs = vec![base.to_path_buf()];
    let resolved = probe_import_base(base, canonical_paths, options, Some(&mut inputs));
    (resolved, inputs)
}

fn probe_import_base(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    mut inputs: Option<&mut Vec<PathBuf>>,
) -> Option<PathBuf> {
    if ImportFileOptions::path_has_typescript_source_extension(base) && base.is_file() {
        return Some(canonical_paths.canonicalize(base));
    }
    if let Some(rewritten) = rewrite_js_to_ts(
        base,
        canonical_paths,
        options.include_jsx,
        inputs.as_deref_mut(),
    ) {
        return Some(rewritten);
    }
    if options.javascript_extension_is_enabled(base) && base.is_file() {
        return Some(canonical_paths.canonicalize(base));
    }
    for ext in options.resolve_extensions() {
        let candidate = append_extension(base, ext);
        record_consulted(inputs.as_deref_mut(), &candidate);
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }
    for ext in options.resolve_extensions() {
        let candidate = base.join(cstr_index(ext));
        record_consulted(inputs.as_deref_mut(), &candidate);
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }
    None
}

fn record_consulted(inputs: Option<&mut Vec<PathBuf>>, candidate: &Path) {
    if let Some(inputs) = inputs {
        inputs.push(candidate.to_path_buf());
    }
}

fn rewrite_js_to_ts(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
    mut inputs: Option<&mut Vec<PathBuf>>,
) -> Option<PathBuf> {
    let name = base.file_name()?.to_str()?;
    let (stem, extensions): (&str, &[&str]) = if let Some(stem) = name.strip_suffix(".mjs") {
        (stem, &[".mts", ".d.mts"])
    } else if let Some(stem) = name.strip_suffix(".cjs") {
        (stem, &[".cts", ".d.cts"])
    } else if let Some(stem) = name.strip_suffix(".jsx") {
        (stem, &[".tsx"])
    } else {
        let stem = name.strip_suffix(".js")?;
        (
            stem,
            if include_jsx {
                &[".ts", ".tsx", ".d.ts"]
            } else {
                &[".ts", ".d.ts"]
            },
        )
    };
    for ext in extensions {
        let candidate = base.with_file_name(cstr!("{stem}{ext}"));
        record_consulted(inputs.as_deref_mut(), &candidate);
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }
    None
}

pub(super) fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

pub(super) fn is_node_modules_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == std::ffi::OsStr::new("node_modules"))
}

fn append_extension(base: &Path, ext: &str) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}{ext}")),
        None => base.to_path_buf(),
    }
}

fn cstr_index(ext: &str) -> String {
    cstr!("index{ext}")
}

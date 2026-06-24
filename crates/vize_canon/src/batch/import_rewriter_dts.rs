//! Redirecting relative re-exports/imports of generated declaration files to
//! their real on-disk path when a non-`.vue` script is materialized into canon.
//!
//! Generated GraphQL `.d.ts` schemas are intentionally never mirrored into
//! `node_modules/.vize/canon` (#2047). A barrel like `types/index.ts` reached
//! transitively from a `.vue` *is* mirrored, but its relative
//! `export * from './codegen/schema'` would then dangle inside the mirror where
//! the schema is absent — dropping the generated module's type identity and
//! producing false `TS2305`/`TS1360` diagnostics on `satisfies` checks (#2227).

use std::path::{Path, PathBuf};

use vize_carton::{String, cstr};

use super::virtual_rewrite::append_extension;

/// Cheap text gate: whether `source` has any relative module specifier worth
/// resolving for the relative-`.d.ts` rewrite. Avoids parsing files that only
/// import bare packages or aliases.
pub(super) fn source_contains_relative_specifier(source: &str) -> bool {
    source.contains("'./")
        || source.contains("\"./")
        || source.contains("'../")
        || source.contains("\"../")
}

/// Rewrite a relative specifier that resolves to a generated `.d.ts` kept on its
/// real path to that real (extensionless) path, so the re-exported identity is
/// preserved inside the mirror.
pub(super) fn rewrite_relative_dts_specifier(
    path: &str,
    source_dir: &Path,
    project_root: &Path,
) -> Option<String> {
    if !(path.starts_with("./") || path.starts_with("../")) {
        return None;
    }
    if path.ends_with(".vue") {
        return None;
    }
    let resolved = resolve_relative_declaration_target(&source_dir.join(path))?;
    // Only generated declarations that live inside the project (and would never
    // be mirrored into canon) need redirecting; node_modules already resolves.
    if !resolved.starts_with(project_root) {
        return None;
    }
    // Drop the `.d.ts`/`.d.mts`/`.d.cts` suffix so the specifier resolves as a
    // module (matching the original extensionless relative import). Importing a
    // declaration extension directly is rejected with TS2846.
    let resolved = strip_declaration_suffix(&resolved);
    Some(cstr!("{}", resolved.display()))
}

/// Resolve a relative specifier base to an existing `.d.ts` file, mirroring
/// TypeScript's `.d.ts` and `index.d.ts` probing. Returns `None` for any target
/// that is not a declaration file so source files keep their relative spelling
/// (the mirror preserves the directory layout for those).
fn resolve_relative_declaration_target(base: &Path) -> Option<PathBuf> {
    if is_declaration_path(base) && base.is_file() {
        return Some(vize_carton::path::canonicalize_non_verbatim(base));
    }
    for ext in [".d.ts", ".d.mts", ".d.cts"] {
        let candidate = append_extension(base, ext);
        if candidate.is_file() {
            return Some(vize_carton::path::canonicalize_non_verbatim(&candidate));
        }
    }
    for ext in [".d.ts", ".d.mts", ".d.cts"] {
        let candidate = base.join(cstr!("index{ext}").as_str());
        if candidate.is_file() {
            return Some(vize_carton::path::canonicalize_non_verbatim(&candidate));
        }
    }
    None
}

/// Strip a `.d.ts`/`.d.mts`/`.d.cts` suffix, leaving the extensionless module
/// path (e.g. `/p/codegen/schema.d.ts` → `/p/codegen/schema`).
fn strip_declaration_suffix(path: &Path) -> PathBuf {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.to_path_buf();
    };
    let stem = name
        .strip_suffix(".d.ts")
        .or_else(|| name.strip_suffix(".d.mts"))
        .or_else(|| name.strip_suffix(".d.cts"));
    match stem {
        Some(stem) => path.with_file_name(stem),
        None => path.to_path_buf(),
    }
}

fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

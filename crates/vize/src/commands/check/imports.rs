//! Transitive resolution of source imports for `vize check` virtual projects.
//!
//! A check run may intentionally report diagnostics for only a subset of
//! sources, but imported local sources still need to be registered so
//! cross-file types resolve like tsc/vue-tsc. This module walks the reachable
//! graph and returns on-disk source files to register.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::imports_aliases::PathAliasResolver;
use super::path_cache::CanonicalPathCache;

/// Source extensions whose imports carry TypeScript types worth pulling into the
/// virtual project, in module-resolution precedence order.
///
/// `.d.ts` is deliberately excluded: ambient declaration files (e.g. a project
/// `shims.d.ts` with a top-level `declare module "vue"`) shadow the real module
/// when registered as program roots, so pulling them in would break `vue`
/// resolution for every file. TypeScript still loads reachable `.d.ts` on demand.
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".vue", ".mts", ".cts"];
const JSX_RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".jsx", ".vue", ".mts", ".cts"];

/// Walk the relative-import graph reachable from `roots` and return the extra
/// on-disk source files that should be registered alongside them. The roots
/// themselves are excluded from the result; every returned path is absolute.
pub(super) fn collect_transitive_local_imports(
    roots: &[PathBuf],
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
    aliases: Option<&PathAliasResolver>,
) -> Vec<PathBuf> {
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue: Vec<PathBuf> = Vec::new();

    // Seed the visited set with the roots so they are never re-registered.
    for root in roots {
        if let Some(absolute) = absolutize(root, cwd, canonical_paths)
            && visited.insert(absolute.clone())
        {
            queue.push(absolute);
        }
    }

    let mut discovered: Vec<PathBuf> = Vec::new();

    while let Some(file) = queue.pop() {
        let Some(dir) = file.parent() else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        // Scan the raw file text directly — the byte scanner only reacts to
        // `import`/`from` string operands, so an SFC's `<template>`/`<style>`
        // are inert and no `.vue` parse is needed on this hot path.
        for specifier in extract_import_specifiers(&source) {
            let resolved = if is_relative_specifier(&specifier) {
                resolve_relative_import(dir, &specifier, canonical_paths, include_jsx)
            } else if Path::new(specifier.as_str()).is_absolute() {
                resolve_import_base(Path::new(specifier.as_str()), canonical_paths, include_jsx)
            } else {
                aliases.and_then(|aliases| {
                    aliases.resolve(
                        &specifier,
                        canonical_paths,
                        include_jsx,
                        resolve_import_base,
                    )
                })
            };
            let Some(resolved) = resolved else {
                continue;
            };
            // Never register an ambient declaration file — its `declare module`
            // statements would shadow real modules as a program root.
            if is_declaration_file(&resolved) || is_node_modules_path(&resolved) {
                continue;
            }
            if visited.insert(resolved.clone()) {
                discovered.push(resolved.clone());
                queue.push(resolved);
            }
        }
    }

    discovered
}

/// Resolve `path` against `cwd` and canonicalize it so duplicate registrations
/// of the same file under different spellings collapse.
fn absolutize(
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

/// Collect module specifiers of `source`'s import/export/dynamic-imports.
///
/// This is a deliberately lightweight byte scan rather than a full parse: the
/// transitive walk runs on every checked file, so an AST per file regressed the
/// benchmark. Over-matching (e.g. an import-like fragment inside a string) is
/// harmless because each specifier is resolved against the filesystem and only
/// real source files are registered.
fn extract_import_specifiers(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while i < len {
        let keyword_len = if matches_keyword(bytes, i, b"from") {
            4
        } else if matches_keyword(bytes, i, b"import") {
            6
        } else {
            i += 1;
            continue;
        };

        let mut j = i + keyword_len;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        // `import('./x')` / `import ( './x' )` — step over the call paren.
        if j < len && bytes[j] == b'(' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }

        if j < len && (bytes[j] == b'"' || bytes[j] == b'\'') {
            let quote = bytes[j];
            let start = j + 1;
            let mut k = start;
            while k < len && bytes[k] != quote {
                k += 1;
            }
            if k < len {
                let specifier = &source[start..k];
                specifiers.push(specifier.to_compact_string());
                i = k + 1;
                continue;
            }
        }
        // `import {` / `import Foo` — no string yet; keep scanning for `from`.
        i += keyword_len;
    }

    specifiers
}

/// Whether `bytes[at..]` begins with `keyword` as a standalone identifier token.
fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if at + keyword.len() > bytes.len() || &bytes[at..at + keyword.len()] != keyword {
        return false;
    }
    let before_ok = at == 0 || !is_identifier_byte(bytes[at - 1]);
    let after = at + keyword.len();
    let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

/// Resolve a relative module specifier against `dir` to an existing on-disk
/// source file, mirroring TypeScript's extension and `index` probing (including
/// the `.js` → `.ts` rewrite used under bundler/Node-ESM resolution).
fn resolve_relative_import(
    dir: &Path,
    specifier: &str,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
) -> Option<PathBuf> {
    resolve_import_base(&dir.join(specifier), canonical_paths, include_jsx)
}

pub(super) fn resolve_import_base(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
) -> Option<PathBuf> {
    // 1. The specifier already points at an existing source file.
    if has_source_extension(base, include_jsx) && base.is_file() {
        return Some(canonical_paths.canonicalize(base));
    }

    // 2. A `.js`/`.mjs`/`.cjs` specifier resolving to its `.ts`/`.tsx` sibling.
    if let Some(rewritten) = rewrite_js_to_ts(base, canonical_paths) {
        return Some(rewritten);
    }

    // 3. Append a source extension: `./types` → `./types.ts`.
    for ext in resolve_extensions(include_jsx) {
        let candidate = append_extension(base, ext);
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }

    // 4. Directory index: `./feature` → `./feature/index.ts`.
    for ext in resolve_extensions(include_jsx) {
        let candidate = base.join(cstr_index(ext));
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }

    None
}

fn rewrite_js_to_ts(base: &Path, canonical_paths: &mut CanonicalPathCache) -> Option<PathBuf> {
    let name = base.file_name()?.to_str()?;
    let stem = name
        .strip_suffix(".js")
        .or_else(|| name.strip_suffix(".mjs"))
        .or_else(|| name.strip_suffix(".cjs"))?;
    for ext in [".ts", ".tsx", ".mts", ".cts"] {
        let candidate = base.with_file_name(cstr!("{stem}{ext}"));
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }
    None
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

fn is_node_modules_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == std::ffi::OsStr::new("node_modules"))
}

fn has_source_extension(path: &Path, include_jsx: bool) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    resolve_extensions(include_jsx)
        .iter()
        .any(|ext| name.ends_with(ext) && name.len() > ext.len())
}

fn resolve_extensions(include_jsx: bool) -> &'static [&'static str] {
    if include_jsx {
        JSX_RESOLVE_EXTENSIONS
    } else {
        RESOLVE_EXTENSIONS
    }
}

/// Append a full extension (e.g. `.d.ts`) to a path's file name without
/// replacing any existing one, so `./a.b` → `./a.b.ts`.
fn append_extension(base: &Path, ext: &str) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}{ext}")),
        None => base.to_path_buf(),
    }
}

#[cfg(test)]
#[path = "imports_tests.rs"]
mod tests;

fn cstr_index(ext: &str) -> String {
    cstr!("index{ext}")
}

//! Transitive resolution of source imports for `vize check` virtual projects.
//!
//! A check run may intentionally report diagnostics for only a subset of
//! sources. This module separates authored files that TypeScript can resolve
//! in place from files that must enter Vize's mirror for Vue import rewriting.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet, String, cstr};

use super::imports_aliases::PathAliasResolver;
use super::path_cache::CanonicalPathCache;

#[path = "imports_registration.rs"]
mod registration;
use registration::non_relative_import_needs_virtual_registration;
#[path = "imports_packages.rs"]
mod packages;
use packages::PackageImportResolver;
#[path = "imports_specifiers.rs"]
mod specifiers;
use specifiers::{extract_import_specifiers, is_relative_specifier};

/// Source extensions whose imports carry TypeScript types worth pulling into the
/// virtual project, in module-resolution precedence order.
///
/// `.d.ts` is deliberately excluded: ambient declaration files (e.g. a project
/// `shims.d.ts` with a top-level `declare module "vue"`) shadow the real module
/// when registered as program roots, so pulling them in would break `vue`
/// resolution for every file. TypeScript still loads reachable `.d.ts` on demand.
#[path = "imports_options.rs"]
mod options;
pub(super) use options::{ImportFileOptions, TransitiveLocalImports};

/// Walk the local import graph reachable from `roots`. The returned sets keep
/// virtual registrations separate from authored in-place diagnostic sources;
/// roots are excluded from both and every returned path is absolute.
pub(super) fn collect_transitive_local_imports(
    roots: &[PathBuf],
    cwd: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: impl Into<ImportFileOptions>,
    aliases: Option<&PathAliasResolver>,
) -> TransitiveLocalImports {
    let options = options.into();
    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut registered: FxHashSet<PathBuf> = FxHashSet::default();
    let mut registration_cache: FxHashMap<PathBuf, bool> = FxHashMap::default();
    let mut packages = PackageImportResolver::default();
    let mut queue: Vec<(PathBuf, bool, bool)> = Vec::new();

    // Seed the visited set with the roots so they are never re-registered.
    for root in roots {
        if let Some(absolute) = absolutize(root, cwd, canonical_paths)
            && visited.insert(absolute.clone())
        {
            registered.insert(absolute.clone());
            queue.push((absolute, true, false));
        }
    }

    let mut registrations: Vec<PathBuf> = Vec::new();
    let mut authored: Vec<PathBuf> = Vec::new();
    let mut virtual_module_aliases: FxHashSet<(String, PathBuf)> = FxHashSet::default();

    while let Some((file, materialized_parent, package_graph)) = queue.pop() {
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
            let relative_specifier = is_relative_specifier(&specifier);
            let absolute_specifier = Path::new(specifier.as_str()).is_absolute();
            let mut package_resolution = false;
            let resolved = if relative_specifier {
                resolve_relative_import(dir, &specifier, canonical_paths, options)
            } else if absolute_specifier {
                resolve_import_base(Path::new(specifier.as_str()), canonical_paths, options)
            } else {
                let aliased = aliases.and_then(|aliases| {
                    aliases.resolve(&specifier, canonical_paths, options, resolve_import_base)
                });
                match aliased {
                    Some(resolved) => Some(resolved),
                    None => {
                        let resolved = packages.resolve(dir, &specifier, canonical_paths, options);
                        package_resolution = resolved.is_some();
                        resolved
                    }
                }
            };
            let Some(resolved) = resolved else {
                continue;
            };
            // Never register an ambient declaration file — its `declare module`
            // statements would shadow real modules as a program root.
            if is_declaration_file(&resolved) || is_node_modules_path(&resolved) {
                continue;
            }
            let needs_registration = if relative_specifier {
                materialized_parent
            } else {
                non_relative_import_needs_virtual_registration(
                    &resolved,
                    canonical_paths,
                    options,
                    aliases,
                    &mut registration_cache,
                )
            };
            let in_package_graph = package_graph || package_resolution;
            if package_resolution && needs_registration {
                virtual_module_aliases.insert((specifier.clone(), resolved.clone()));
            }
            let first_visit = visited.insert(resolved.clone());
            let first_registration = needs_registration && registered.insert(resolved.clone());
            if first_visit {
                authored.push(resolved.clone());
            }
            // A bare package route is registered by Canon after the project
            // root is fixed. Adding it (or its relative descendants) to the
            // user's roots here would widen the project to the workspace and
            // defeat the external mirror.
            if first_registration && !in_package_graph {
                registrations.push(resolved.clone());
            }
            if first_visit || first_registration {
                queue.push((resolved, needs_registration, in_package_graph));
            }
        }
    }

    let mut virtual_module_aliases = virtual_module_aliases.into_iter().collect::<Vec<_>>();
    virtual_module_aliases.sort();
    TransitiveLocalImports {
        registrations,
        authored,
        virtual_module_aliases,
    }
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

/// Resolve a relative module specifier against `dir` to an existing on-disk
/// source file, mirroring TypeScript's extension and `index` probing (including
/// the `.js` → `.ts` rewrite used under bundler/Node-ESM resolution).
fn resolve_relative_import(
    dir: &Path,
    specifier: &str,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> Option<PathBuf> {
    resolve_import_base(&dir.join(specifier), canonical_paths, options)
}

pub(super) fn resolve_import_base(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> Option<PathBuf> {
    // 1. The specifier already points at an existing TS/Vue source file.
    if ImportFileOptions::path_has_typescript_source_extension(base) && base.is_file() {
        return Some(canonical_paths.canonicalize(base));
    }

    // 2. A `.js`/`.jsx`/`.mjs`/`.cjs` specifier resolving to its TS sibling.
    if let Some(rewritten) = rewrite_js_to_ts(base, canonical_paths, options.include_jsx) {
        return Some(rewritten);
    }

    // 3. Under allowJs (or the JSX feature), keep an existing JS-family file.
    if options.javascript_extension_is_enabled(base) && base.is_file() {
        return Some(canonical_paths.canonicalize(base));
    }

    // 4. Append a source extension: `./types` → `./types.ts`.
    for ext in options.resolve_extensions() {
        let candidate = append_extension(base, ext);
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }

    // 5. Directory index: `./feature` → `./feature/index.ts`.
    for ext in options.resolve_extensions() {
        let candidate = base.join(cstr_index(ext));
        if candidate.is_file() {
            return Some(canonical_paths.canonicalize(&candidate));
        }
    }

    None
}

fn rewrite_js_to_ts(
    base: &Path,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
) -> Option<PathBuf> {
    let name = base.file_name()?.to_str()?;
    let (stem, extensions): (&str, &[&str]) = if let Some(stem) = name.strip_suffix(".mjs") {
        (stem, &[".mts"])
    } else if let Some(stem) = name.strip_suffix(".cjs") {
        (stem, &[".cts"])
    } else if let Some(stem) = name.strip_suffix(".jsx") {
        (stem, &[".tsx"])
    } else if let Some(stem) = name.strip_suffix(".js") {
        (
            stem,
            if include_jsx {
                &[".ts", ".tsx"]
            } else {
                &[".ts"]
            },
        )
    } else {
        return None;
    };
    for ext in extensions {
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

/// Append a full extension (e.g. `.d.ts`) to a path's file name without
/// replacing any existing one, so `./a.b` → `./a.b.ts`.
fn append_extension(base: &Path, ext: &str) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}{ext}")),
        None => base.to_path_buf(),
    }
}

#[cfg(test)]
#[path = "imports_generated_tests.rs"]
mod generated_tests;
#[cfg(test)]
#[path = "imports_js_tests.rs"]
mod js_tests;
#[cfg(test)]
#[path = "imports_tests.rs"]
mod tests;

fn cstr_index(ext: &str) -> String {
    cstr!("index{ext}")
}

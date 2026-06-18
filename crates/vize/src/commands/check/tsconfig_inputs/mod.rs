//! `tsconfig.json`-driven default input collection for `vize check`.
//!
//! When users run `vize check` without explicit paths, we should follow the
//! project's configured `files` / `include` / `exclude` fields instead of
//! recursively scanning every TypeScript file under the working directory.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

mod ambient;
mod collect;
mod glob;
mod jsonc;
mod loader;
mod matching;
mod spec;

#[cfg(test)]
mod tests;

pub(crate) use ambient::collect_ambient_declaration_files;
pub(crate) use collect::resolve_tsconfig_for_files;
pub(super) use jsonc::parse_jsonc_value;
pub(crate) use loader::{TsconfigInputCache, load_tsconfig_declaration_options};
pub(super) use loader::{read_extends_entries, resolve_extended_tsconfig};
pub(crate) use spec::TsconfigDeclarationOptions;

use collect::{
    collect_supported_files_for_include_roots, collect_supported_files_with_options,
    explicit_hidden_include_roots,
};
use glob::{default_exclude_specs, normalize_input_path};
use loader::collect_tsconfig_project_paths;
use matching::{
    SupportedFileOptions, is_nuxt_import_manifest_path, is_supported_check_file_with_options,
};
use spec::{FileCollectionOptions, GlobSpec, TsconfigInputSpec};

const TARGET_DIR: &str = "target";
const NODE_MODULES_DIR: &str = "node_modules";
const VIZE_CACHE_DIR: &str = ".vize";

pub(crate) fn collect_default_check_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) -> Vec<PathBuf> {
    collect_default_check_files_inner(project_root, tsconfig_path, false, include_jsx, cache)
}

fn collect_default_check_files_inner(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    include_hidden_tsconfig_roots: bool,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) -> Vec<PathBuf> {
    let Some(tsconfig_path) = tsconfig_path else {
        return collect_supported_files_with_options(
            project_root,
            &[],
            &[],
            FileCollectionOptions {
                include_hidden: false,
                include_jsx,
            },
        );
    };

    let mut files = Vec::new();
    let mut seen = FxHashSet::default();
    for tsconfig_path in collect_tsconfig_project_paths(tsconfig_path) {
        collect_default_check_files_for_tsconfig(
            project_root,
            &tsconfig_path,
            include_hidden_tsconfig_roots,
            include_jsx,
            cache,
            &mut files,
            &mut seen,
        );
    }

    files.sort();
    files
}

fn collect_default_check_files_for_tsconfig(
    project_root: &Path,
    tsconfig_path: &Path,
    include_hidden_tsconfig_roots: bool,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    let default_spec = TsconfigInputSpec::default();
    let spec = cache.load(tsconfig_path).unwrap_or(&default_spec);

    for file in &spec.files {
        let resolved = normalize_input_path(&file.resolve());
        if resolved.is_file()
            && is_supported_check_file_with_options(&resolved, SupportedFileOptions { include_jsx })
            && !is_nuxt_import_manifest_path(&resolved)
            && seen.insert(resolved.clone())
        {
            files.push(resolved);
        }
    }

    let default_base_dir = tsconfig_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_root.to_path_buf());

    let default_includes;
    let includes: &[GlobSpec] = if !spec.has_includes && !spec.has_files && files.is_empty() {
        default_includes = GlobSpec::new(&default_base_dir, "**/*")
            .into_iter()
            .collect::<Vec<_>>();
        &default_includes
    } else {
        &spec.includes
    };

    let default_excludes;
    let excludes: &[GlobSpec] = if !spec.has_excludes {
        default_excludes = default_exclude_specs(&default_base_dir);
        &default_excludes
    } else {
        &spec.excludes
    };

    if !includes.is_empty() {
        let collected = collect_supported_files_for_include_roots(
            project_root,
            includes,
            excludes,
            FileCollectionOptions {
                include_hidden: false,
                include_jsx,
            },
        );
        for path in collected {
            if seen.insert(path.clone()) {
                files.push(path);
            }
        }
        if include_hidden_tsconfig_roots {
            for root in explicit_hidden_include_roots(project_root, includes) {
                for path in collect_supported_files_with_options(
                    &root,
                    includes,
                    excludes,
                    FileCollectionOptions {
                        include_hidden: true,
                        include_jsx,
                    },
                ) {
                    if seen.insert(path.clone()) {
                        files.push(path);
                    }
                }
            }
        }
    }
}

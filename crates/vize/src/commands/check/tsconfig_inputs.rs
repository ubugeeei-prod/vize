//! `tsconfig.json`-driven default input collection for `vize check`.
//!
//! When users run `vize check` without explicit paths, we should follow the
//! project's configured `files` / `include` / `exclude` fields instead of
//! recursively scanning every TypeScript file under the working directory.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use vize_s0::FxHashSet;

mod ambient;
mod collect;
mod glob;
mod implicit_exclude;
mod jsonc;
mod loader;
mod matching;
mod ownership;
mod spec;
mod type_references;

#[cfg(test)]
mod default_exclude_tests;
#[cfg(test)]
mod hidden_include_tests;
#[cfg(test)]
mod nuxt_manifest_tests;
#[cfg(test)]
mod ownership_shared_tests;
#[cfg(test)]
mod package_folder_tests;
#[cfg(test)]
mod tests;

pub(crate) use ambient::{
    collect_ambient_declaration_files, collect_hidden_ambient_declaration_files,
};
pub(super) use jsonc::parse_jsonc_value;
pub(crate) use loader::{TsconfigInputCache, load_tsconfig_declaration_options};
pub(super) use loader::{read_extends_entries, resolve_extended_tsconfig};
pub(crate) use ownership::{resolve_tsconfig_for_files, resolve_tsconfig_program_inputs};
pub(crate) use spec::TsconfigDeclarationOptions;
pub(crate) use type_references::{
    collect_tsconfig_type_packages, reference_type_packages, resolve_type_package_declaration_files,
};

use collect::{
    collect_supported_files_for_include_roots, collect_supported_files_with_options,
    explicit_hidden_include_roots, explicit_hidden_pattern_roots,
};
use glob::normalize_input_path;
use matching::{
    SupportedFileOptions, is_declaration_path, is_generated_codegen_declaration_path,
    is_nuxt_import_manifest_path, is_supported_check_file_with_options,
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

pub(crate) fn tsconfig_allows_js(tsconfig_path: &Path, cache: &mut TsconfigInputCache) -> bool {
    cache.project_allows_js(tsconfig_path)
}

/// Whether the root project or any transitively referenced project accepts
/// JavaScript inputs. Explicit path collection uses this only as a broad
/// extension gate; ownership resolution later selects the exact project and
/// applies that project's own `allowJs` value.
pub(crate) fn tsconfig_project_graph_allows_js(
    tsconfig_path: &Path,
    cache: &mut TsconfigInputCache,
) -> bool {
    cache
        .project_paths(tsconfig_path)
        .iter()
        .any(|project| tsconfig_allows_js(project, cache))
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
                include_js: false,
                include_jsx,
            },
        );
    };

    let mut files = Vec::new();
    let mut seen = FxHashSet::default();
    for (index, tsconfig_path) in cache.project_paths(tsconfig_path).iter().enumerate() {
        // `tsc -p` expands `references` into checkable programs only for a
        // solution-style root — one that contributes no inputs of its own
        // (create-vue's `"files": []` shell, Nuxt's generated root). A root
        // that names its own program keeps `-p` semantics: its `include`,
        // `files`, and `exclude` alone decide the file set, and a referenced
        // workspace — possibly deliberately `exclude`d — is a separate
        // project, not part of this check (#4965).
        if index == 1 && !files.is_empty() {
            break;
        }
        collect_default_check_files_for_tsconfig(
            project_root,
            tsconfig_path,
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
    let include_js = spec.allow_js.unwrap_or(false);

    for file in &spec.files {
        let resolved = normalize_input_path(&file.resolve());
        if resolved.is_file()
            && is_supported_check_file_with_options(
                &resolved,
                SupportedFileOptions {
                    include_js,
                    include_jsx,
                },
            )
            && !is_nuxt_import_manifest_path(&resolved)
            && !is_generated_codegen_declaration_path(&resolved)
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

    let excludes = spec.effective_excludes();
    let excludes: &[GlobSpec] = &excludes;

    if !includes.is_empty() {
        let collected = collect_supported_files_for_include_roots(
            project_root,
            includes,
            excludes,
            FileCollectionOptions {
                include_hidden: false,
                include_js,
                include_jsx,
            },
        );
        for path in collected {
            if !is_generated_codegen_declaration_path(&path) && seen.insert(path.clone()) {
                files.push(path);
            }
        }
        // A tsconfig that names a dot-directory literally — VitePress
        // `.vitepress`, Storybook `.storybook` — puts those files in the
        // program, so they must be checked, not just scanned for ambient
        // declarations.
        let hidden_roots = if include_hidden_tsconfig_roots {
            explicit_hidden_include_roots(project_root, includes)
        } else {
            explicit_hidden_pattern_roots(includes)
        };
        for root in hidden_roots {
            for path in collect_supported_files_with_options(
                &root,
                includes,
                excludes,
                FileCollectionOptions {
                    include_hidden: true,
                    include_js,
                    include_jsx,
                },
            ) {
                // Declaration files under a hidden root are ambient program
                // inputs, not check sources: `collect_hidden_ambient_declaration_files`
                // already loads them, and reporting a generated
                // `.nuxt/components.d.cts` as a checked file diverges from
                // vue-tsc, which surfaces no diagnostics for it.
                if !include_hidden_tsconfig_roots && is_declaration_path(&path) {
                    continue;
                }
                if !is_generated_codegen_declaration_path(&path) && seen.insert(path.clone()) {
                    files.push(path);
                }
            }
        }
    }
}

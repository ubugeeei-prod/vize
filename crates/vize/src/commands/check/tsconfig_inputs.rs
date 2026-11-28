//! `tsconfig.json`-driven default input collection for `vize check`.
//!
//! When users run `vize check` without explicit paths, we should follow the
//! project's configured `files` / `include` / `exclude` fields instead of
//! recursively scanning every TypeScript file under the working directory.

#![expect(
    clippy::disallowed_types,
    reason = "tsconfig text and globs round-trip through std `String` APIs (fs, serde_json, glob)"
)]

use std::path::{Path, PathBuf};

use vize_l0::FxHashSet;

mod ambient;
mod chain;
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
mod diamond_tests;
#[cfg(test)]
mod hidden_include_tests;
#[cfg(test)]
mod ignore_file_tests;
#[cfg(test)]
mod literal_bracket_tests;
#[cfg(test)]
mod nuxt_manifest_tests;
#[cfg(test)]
mod ownership_shared_tests;
#[cfg(test)]
mod package_folder_tests;
#[cfg(test)]
mod tests;

pub(crate) use ambient::{
    AugmentationImports, collect_ambient_declaration_files,
    collect_content_mapper_ambient_declarations, collect_declaration_reference_graph,
    collect_hidden_ambient_declaration_files, collect_selected_nuxt_import_manifests,
    collect_tsconfig_type_declaration_files,
};
pub(super) use jsonc::parse_jsonc_value;
pub(crate) use loader::{TsconfigInputCache, load_tsconfig_declaration_options};
pub(super) use loader::{read_extends_entries, resolve_extended_tsconfig};
pub(crate) use ownership::{
    TsconfigProgramInputs, partition_program_owned_sources, referenced_projects,
    resolve_tsconfig_for_files, resolve_tsconfig_program_inputs, retain_program_owned_sources,
};
pub(crate) use spec::TsconfigDeclarationOptions;
pub(crate) use type_references::{
    collect_tsconfig_type_packages, reference_type_packages, resolve_type_package_declaration_files,
};

#[cfg(test)]
pub(crate) use collect::walk_probe;
use collect::{
    collect_supported_files_for_include_roots, collect_supported_files_with_options,
    explicit_hidden_include_roots, explicit_hidden_pattern_roots,
};
pub(super) use glob::normalize_input_path;
use matching::{
    SupportedFileOptions, is_declaration_path, is_generated_codegen_declaration_path,
    is_nuxt_import_manifest_path, is_supported_check_file_with_options,
};
use spec::{
    FileCollectionOptions, GlobSpec, InputSelection, ProgramInputOptions, TsconfigInputSpec,
};

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

/// Whether the selected program declares the global `useTemplateRef` as
/// Vue's own export. Declaration reference edges are part of the program too.
pub(crate) fn declares_vue_template_ref_global(files: &[PathBuf]) -> bool {
    collect_declaration_reference_graph(files)
        .iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .any(|source| vize_canon::batch::declares_vue_template_ref_global(&source))
}

/// Whether the program's `#imports` alias resolves to a module that
/// re-exports Vue's own `useTemplateRef` (Nuxt's generated `imports.d.ts`).
pub(crate) fn imports_alias_reexports_vue_template_ref(tsconfig_path: &Path) -> bool {
    super::imports_aliases::imports_alias_reexports_vue_template_ref(tsconfig_path)
}

/// The resolved `#imports` file, for hosts that watch it, and whether it
/// re-exports Vue's own `useTemplateRef`.
pub(crate) fn imports_alias_vue_template_ref(tsconfig_path: &Path) -> Option<(PathBuf, bool)> {
    super::imports_aliases::imports_alias_vue_template_ref(tsconfig_path)
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
                respect_ignore_files: true,
                include_hidden: false,
                include_js: false,
                include_jsx,
                selection: InputSelection::All,
            },
        );
    };
    let options = ProgramInputOptions {
        include_hidden_tsconfig_roots,
        include_jsx,
        selection: InputSelection::All,
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
            options,
            cache,
            &mut files,
            &mut seen,
        );
    }

    files.sort();
    files
}

/// Whether a tsconfig program selects any input of its own, which keeps
/// `tsc -p` from expanding its references (see
/// [`collect_default_check_files_inner`]). The walk stops at the first input.
fn tsconfig_selects_any_input(
    project_root: &Path,
    tsconfig_path: &Path,
    include_hidden_tsconfig_roots: bool,
    cache: &mut TsconfigInputCache,
) -> bool {
    let mut files = Vec::new();
    collect_default_check_files_for_tsconfig(
        project_root,
        tsconfig_path,
        ProgramInputOptions {
            include_hidden_tsconfig_roots,
            include_jsx: false,
            selection: InputSelection::Any,
        },
        cache,
        &mut files,
        &mut FxHashSet::default(),
    );
    !files.is_empty()
}

fn collect_default_check_files_for_tsconfig(
    project_root: &Path,
    tsconfig_path: &Path,
    options: ProgramInputOptions,
    cache: &mut TsconfigInputCache,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    let ProgramInputOptions {
        include_hidden_tsconfig_roots,
        include_jsx,
        selection,
    } = options;
    let default_spec = TsconfigInputSpec::default();
    let spec = cache.load(tsconfig_path).unwrap_or(&default_spec);
    let include_js = spec.allow_js.unwrap_or(false);
    let selected_enough =
        |files: &Vec<PathBuf>| selection == InputSelection::Any && !files.is_empty();

    for file in &spec.files {
        if selected_enough(files) {
            return;
        }
        let named = file.resolve();
        // `tsc` reads a root by the name the tsconfig spells, so the name
        // alone rules out an ordinary input before any filesystem access. The
        // canonical path must still name a declaration as well.
        if selection == InputSelection::Declarations && !is_declaration_path(&named) {
            continue;
        }
        let resolved = normalize_input_path(&named);
        if vize_carton::path::is_git_metadata_path(&resolved)
            || (selection == InputSelection::Declarations && !is_declaration_path(&resolved))
        {
            continue;
        }
        #[cfg(test)]
        collect::walk_probe::record(&resolved);
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
    let includes: &[GlobSpec] = if !spec.has_includes && !spec.has_files {
        default_includes = GlobSpec::include(&default_base_dir, "**/*")
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
                respect_ignore_files: false,
                include_hidden: false,
                include_js,
                include_jsx,
                selection,
            },
        );
        for path in collected {
            if !is_generated_codegen_declaration_path(&path) && seen.insert(path.clone()) {
                files.push(path);
            }
        }
        if selected_enough(files) {
            return;
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
            if selected_enough(files) {
                return;
            }
            for path in collect_supported_files_with_options(
                &root,
                includes,
                excludes,
                FileCollectionOptions {
                    respect_ignore_files: false,
                    include_hidden: true,
                    include_js,
                    include_jsx,
                    selection,
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

//! The declaration graph that decides content-mapper ownership.

use std::path::{Path, PathBuf};

use vize_l0::FxHashSet;

use super::super::collect::explicit_hidden_include_roots;
use super::super::glob::normalize_input_path;
use super::super::loader::TsconfigInputCache;
use super::super::matching::{is_nuxt_import_manifest_path, matches_tsconfig_patterns};
use super::super::spec::{InputSelection, ProgramInputOptions};
use super::super::{collect_default_check_files_for_tsconfig, tsconfig_selects_any_input};
use super::{collect_declaration_reference_graph_reading, collect_tsconfig_type_declaration_files};

/// The declaration graph a content-mapper project owns.
pub(crate) struct ContentMapperAmbientDeclarations {
    /// Every declaration of the graph, sorted.
    pub files: Vec<PathBuf>,
    /// Whether one of them declares Vue's global `useTemplateRef`, decided
    /// while the graph walk reads it.
    pub declares_vue_template_ref: bool,
}

/// Collect the declaration graph that decides content-mapper ownership.
///
/// Under a content mapper `tsc` loads the program itself, so every configured
/// declaration is in scope together with everything it references, including
/// Nuxt's import manifests and reference-only manifests such as
/// `.nuxt/nuxt.d.ts`. `vize check` keeps those out of its forwarded ambient
/// roots, but that forwarding filter must not hide a manifest's Vue-owned
/// `useTemplateRef` from the mapper (#13093). Every returned file can change
/// the result and is therefore watched.
///
/// This runs inside `openProject`, before the first transform reply, and the
/// host fingerprints every transform with the identity it decides, so it cannot
/// be deferred. Ordinary source modules are rejected by their directory-entry
/// name: they are never matched against patterns, stat'd, or opened (#13092).
pub(crate) fn collect_content_mapper_ambient_declarations(
    project_root: &Path,
    tsconfig_path: &Path,
    cache: &mut TsconfigInputCache,
) -> ContentMapperAmbientDeclarations {
    let project_root = normalize_input_path(project_root);
    let mut roots = Vec::new();
    let mut seen = FxHashSet::default();
    let mut manifests = Vec::new();
    // One pass over the same program projects `collect_default_check_files_inner`
    // walks: a solution-style root expands into its references, and each
    // project contributes the Nuxt manifests its own `files`/`include` select.
    let projects = cache.project_paths(tsconfig_path);
    for (index, project) in projects.iter().enumerate() {
        // The root keeps `tsc -p` semantics when it selects any input of its
        // own; only a root without declarations needs to look for one.
        if index == 1
            && (!roots.is_empty()
                || projects.first().is_some_and(|root| {
                    tsconfig_selects_any_input(&project_root, root, true, cache)
                }))
        {
            break;
        }
        collect_default_check_files_for_tsconfig(
            &project_root,
            project,
            ProgramInputOptions {
                include_hidden_tsconfig_roots: true,
                include_jsx: false,
                selection: InputSelection::Declarations,
            },
            cache,
            &mut roots,
            &mut seen,
        );
        manifests.extend(collect_selected_nuxt_import_manifests(
            &project_root,
            project,
            cache,
        ));
    }
    roots.extend(collect_tsconfig_type_declaration_files(
        &project_root,
        Some(tsconfig_path),
    ));
    roots.extend(manifests);
    // Each declaration is read once: the walk that follows its references
    // also decides the template-ref ownership.
    let mut declares_vue_template_ref = false;
    let mut files = collect_declaration_reference_graph_reading(&roots, |content| {
        declares_vue_template_ref = declares_vue_template_ref
            || vize_canon::batch::declares_vue_template_ref_global(content);
    })
    .into_iter()
    .filter(|path| path.is_file())
    .collect::<Vec<_>>();
    files.sort();
    ContentMapperAmbientDeclarations {
        files,
        declares_vue_template_ref,
    }
}

/// Nuxt import manifests the tsconfig itself selects through `files` or
/// `include`.
///
/// Input collection never returns these generated declarations, yet `tsc`
/// loads a manifest a tsconfig names. A manifest lives directly in a `.nuxt`
/// directory or in its `types` child, and an include reaches a dot-directory
/// only through a literal segment, so the candidates are the manifest paths
/// of each `.nuxt` directory on an explicit hidden include root, kept when the
/// tsconfig's own patterns select them.
pub(crate) fn collect_selected_nuxt_import_manifests(
    project_root: &Path,
    tsconfig_path: &Path,
    cache: &mut TsconfigInputCache,
) -> Vec<PathBuf> {
    let Some(spec) = cache.load(tsconfig_path).cloned() else {
        return Vec::new();
    };
    // The spelled name rules out every other `files` entry before it is
    // canonicalized or stat'd; the canonical path must still be a manifest.
    let mut manifests = spec
        .files
        .iter()
        .map(|file| file.resolve())
        .filter(|named| is_nuxt_import_manifest_path(named))
        .map(|named| normalize_input_path(&named))
        .filter(|path| {
            #[cfg(test)]
            super::super::collect::walk_probe::record(path);
            is_nuxt_import_manifest_path(path) && path.is_file()
        })
        .collect::<Vec<_>>();
    let excludes = spec.effective_excludes();
    for root in explicit_hidden_include_roots(project_root, &spec.includes) {
        let nuxt_dirs = root
            .ancestors()
            .filter(|dir| dir.file_name().is_some_and(|name| name == ".nuxt"));
        for nuxt_dir in nuxt_dirs {
            for dir in [nuxt_dir.to_path_buf(), nuxt_dir.join("types")] {
                for name in ["imports.d.ts", "imports.d.mts", "imports.d.cts"] {
                    let candidate = dir.join(name);
                    if candidate.is_file()
                        && matches_tsconfig_patterns(&candidate, &spec.includes, &excludes)
                        && !manifests.contains(&candidate)
                    {
                        manifests.push(candidate);
                    }
                }
            }
        }
    }
    manifests
}

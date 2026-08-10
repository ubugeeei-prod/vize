use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{
    CollectedRoots,
    collect::path_is_inside_root,
    ignores::{CheckIgnoreSet, retain_unignored},
};
use crate::commands::check::{
    imports::{ImportFileOptions, TransitiveLocalImports, collect_transitive_local_imports},
    imports_aliases::PathAliasResolver,
    path_cache::CanonicalPathCache,
    tsconfig_inputs::{
        TsconfigInputCache, collect_ambient_declaration_files, collect_default_check_files,
        collect_hidden_ambient_declaration_files,
    },
};

pub(super) struct ExplicitAmbientImportContext<'a> {
    project_root: &'a Path,
    cwd: &'a Path,
    tsconfig_path: &'a Path,
    explicit_input_root: &'a Path,
    import_options: ImportFileOptions,
}

pub(super) struct RegisteredLocalImports {
    pub(super) authored: Vec<PathBuf>,
    pub(super) virtual_module_aliases: Vec<(vize_carton::String, PathBuf)>,
}

impl<'a> ExplicitAmbientImportContext<'a> {
    pub(super) fn new(
        project_root: &'a Path,
        cwd: &'a Path,
        tsconfig_path: &'a Path,
        explicit_input_root: &'a Path,
        import_options: ImportFileOptions,
    ) -> Self {
        Self {
            project_root,
            cwd,
            tsconfig_path,
            explicit_input_root,
            import_options,
        }
    }
}

pub(super) fn collect_default_run_files(
    project_root: &Path,
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    import_options: ImportFileOptions,
    tsconfig_input_cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
    check_ignore_set: Option<&CheckIgnoreSet>,
) -> CollectedRoots {
    let mut files = collect_default_check_files(
        project_root,
        tsconfig_path,
        import_options.include_jsx,
        tsconfig_input_cache,
    );
    retain_unignored(&mut files, check_ignore_set);
    let inputs = files.clone();
    let mut reported_files = canonical_file_set(&files, canonical_paths);
    let discovered = register_transitive_local_imports(
        &mut files,
        cwd,
        tsconfig_path,
        import_options,
        canonical_paths,
        None,
        false,
    );
    reported_files.extend(canonical_file_set(&discovered.authored, canonical_paths));
    let mut virtual_module_aliases = discovered.virtual_module_aliases;
    register_ambient_declaration_files(
        &mut files,
        project_root,
        tsconfig_path,
        tsconfig_input_cache,
    );
    // Imports reached only through hidden ambient declarations provide type
    // context, but are not authored members of the checked program.
    let hidden_discovered = register_transitive_local_imports(
        &mut files,
        cwd,
        tsconfig_path,
        import_options,
        canonical_paths,
        None,
        false,
    );
    virtual_module_aliases.extend(hidden_discovered.virtual_module_aliases);
    virtual_module_aliases.sort();
    virtual_module_aliases.dedup();

    CollectedRoots {
        files,
        inputs,
        reported: reported_files,
        virtual_module_aliases,
    }
}

pub(super) fn register_ambient_declaration_files(
    files: &mut Vec<PathBuf>,
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    tsconfig_input_cache: &mut TsconfigInputCache,
) {
    for path in
        collect_hidden_ambient_declaration_files(project_root, tsconfig_path, tsconfig_input_cache)
    {
        if !files.contains(&path) {
            files.push(path);
        }
    }
}

pub(super) fn register_explicit_ambient_imports(
    files: &mut Vec<PathBuf>,
    context: ExplicitAmbientImportContext<'_>,
    tsconfig_input_cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
) {
    let keep_package_local =
        super::resolve::project_root_has_package_boundary(context.project_root);
    let ambient_declarations = collect_ambient_declaration_files(
        context.project_root,
        Some(context.tsconfig_path),
        tsconfig_input_cache,
    )
    .into_iter()
    .filter(|path| !keep_package_local || path.starts_with(context.project_root))
    .collect::<Vec<_>>();
    files.extend(collect_transitive_local_imports_from(
        &ambient_declarations,
        context.cwd,
        Some(context.tsconfig_path),
        context.import_options,
        canonical_paths,
        Some(context.explicit_input_root),
        true,
    ));
    files.extend(ambient_declarations);
    files.sort();
    files.dedup();
}

pub(super) fn canonical_file_set(
    files: &[PathBuf],
    canonical_paths: &mut CanonicalPathCache,
) -> FxHashSet<PathBuf> {
    files
        .iter()
        .map(|path| canonical_paths.canonicalize(path))
        .collect()
}

pub(super) fn register_transitive_local_imports(
    files: &mut Vec<PathBuf>,
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    import_options: ImportFileOptions,
    canonical_paths: &mut CanonicalPathCache,
    explicit_input_root: Option<&Path>,
    validate_inputs: bool,
) -> RegisteredLocalImports {
    let discovered =
        collect_local_imports(files, cwd, tsconfig_path, import_options, canonical_paths);
    // The explicit-root boundary constrains user-selected roots and files that
    // enter Vize's mirror. It must not hide authored modules that TypeScript
    // legitimately resolves in place outside that boundary.
    let TransitiveLocalImports {
        registrations,
        authored,
        virtual_module_aliases,
    } = discovered;
    append_local_imports(files, registrations, explicit_input_root, validate_inputs);
    RegisteredLocalImports {
        authored,
        virtual_module_aliases,
    }
}

pub(super) fn collect_transitive_local_imports_from(
    roots: &[PathBuf],
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    import_options: ImportFileOptions,
    canonical_paths: &mut CanonicalPathCache,
    explicit_input_root: Option<&Path>,
    validate_inputs: bool,
) -> Vec<PathBuf> {
    collect_local_imports(roots, cwd, tsconfig_path, import_options, canonical_paths)
        .registrations
        .into_iter()
        .filter(|path| local_import_is_allowed(path, explicit_input_root, validate_inputs))
        .collect()
}

fn collect_local_imports(
    roots: &[PathBuf],
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    import_options: ImportFileOptions,
    canonical_paths: &mut CanonicalPathCache,
) -> TransitiveLocalImports {
    let aliases = PathAliasResolver::from_tsconfig(tsconfig_path);
    collect_transitive_local_imports(roots, cwd, canonical_paths, import_options, Some(&aliases))
}

fn append_local_imports(
    files: &mut Vec<PathBuf>,
    discovered: Vec<PathBuf>,
    explicit_input_root: Option<&Path>,
    validate_inputs: bool,
) -> Vec<PathBuf> {
    let mut appended = Vec::new();
    for path in discovered {
        if local_import_is_allowed(&path, explicit_input_root, validate_inputs)
            && !files.contains(&path)
        {
            files.push(path.clone());
            appended.push(path);
        }
    }
    files.sort();
    files.dedup();
    appended
}

fn local_import_is_allowed(
    path: &Path,
    explicit_input_root: Option<&Path>,
    validate_inputs: bool,
) -> bool {
    !validate_inputs || explicit_input_root.is_none_or(|root| path_is_inside_root(root, path))
}

#[cfg(test)]
#[path = "default_imports_tests.rs"]
mod tests;

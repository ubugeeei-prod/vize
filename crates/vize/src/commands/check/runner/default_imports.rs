use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{
    collect::path_is_inside_root,
    ignores::{CheckIgnoreSet, retain_unignored},
};
use crate::commands::check::{
    imports::collect_transitive_local_imports,
    imports_aliases::PathAliasResolver,
    path_cache::CanonicalPathCache,
    tsconfig_inputs::{TsconfigInputCache, collect_default_check_files},
};

pub(super) fn collect_default_run_files(
    project_root: &Path,
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    include_jsx: bool,
    tsconfig_input_cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
    check_ignore_set: Option<&CheckIgnoreSet>,
) -> (Vec<PathBuf>, FxHashSet<PathBuf>) {
    let mut files = collect_default_check_files(
        project_root,
        tsconfig_path,
        include_jsx,
        tsconfig_input_cache,
    );
    retain_unignored(&mut files, check_ignore_set);
    let reported_files = canonical_file_set(&files, canonical_paths);
    register_transitive_local_imports(
        &mut files,
        cwd,
        tsconfig_path,
        include_jsx,
        canonical_paths,
        None,
        false,
    );

    (files, reported_files)
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
    include_jsx: bool,
    canonical_paths: &mut CanonicalPathCache,
    explicit_input_root: Option<&Path>,
    validate_inputs: bool,
) {
    let aliases = PathAliasResolver::from_tsconfig(tsconfig_path);
    for path in
        collect_transitive_local_imports(files, cwd, canonical_paths, include_jsx, Some(&aliases))
    {
        let inside_allowed = !validate_inputs
            || explicit_input_root.is_none_or(|root| path_is_inside_root(root, &path));
        if inside_allowed && !files.contains(&path) {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vize_carton::path::canonicalize_non_verbatim;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(format!(
                "check-runner-{name}-{}-{case_id}",
                std::process::id()
            ))
    }

    #[test]
    fn default_tsconfig_run_registers_transitive_imports_outside_include_for_type_resolution() {
        let project_root = unique_case_dir("default-transitive-imports");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join("inside")).unwrap();
        std::fs::create_dir_all(project_root.join("outside")).unwrap();
        std::fs::write(
            project_root.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["inside/**/*.ts"]
}"#,
        )
        .unwrap();
        std::fs::write(
            project_root.join("inside/use.ts"),
            r#"import { ITEMS } from '../outside/lib'

export const r = ITEMS.map(({ code, name }) => `${code}:${name}`)
"#,
        )
        .unwrap();
        std::fs::write(
            project_root.join("outside/lib.ts"),
            "export const ITEMS = [{ code: 'en', name: 'English' }, { code: 'ru', name: 'Russian' }]\n",
        )
        .unwrap();

        let mut tsconfig_input_cache = super::TsconfigInputCache::default();
        let mut canonical_paths = super::CanonicalPathCache::default();
        let (files, reported_files) = super::collect_default_run_files(
            &project_root,
            &project_root,
            Some(&project_root.join("tsconfig.json")),
            false,
            &mut tsconfig_input_cache,
            &mut canonical_paths,
            None,
        );

        let included_file = canonicalize_non_verbatim(&project_root.join("inside/use.ts"));
        let transitive_file = canonicalize_non_verbatim(&project_root.join("outside/lib.ts"));

        assert!(files.contains(&included_file));
        assert!(files.contains(&transitive_file));
        assert!(reported_files.contains(&included_file));
        assert!(
            !reported_files.contains(&transitive_file),
            "outside-include imports are registered for types, not reported"
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }
}

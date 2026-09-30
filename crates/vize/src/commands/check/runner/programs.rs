//! Splitting one check invocation into effective TypeScript programs.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use vize_l0::FxHashSet;

use super::default_imports::canonical_file_set;
use super::resolve::find_nearest_tsconfig_dir;
use crate::commands::check::{
    path_cache::CanonicalPathCache,
    tsconfig_inputs::{TsconfigInputCache, resolve_tsconfig_program_inputs},
};

pub(super) struct ProgramCandidate {
    pub(super) files: Vec<PathBuf>,
    pub(super) inputs: Vec<PathBuf>,
    pub(super) reported: FxHashSet<PathBuf>,
    pub(super) package_routes: Vec<vize_canon::PackageRouteBinding>,
    pub(super) tsconfig_path: Option<PathBuf>,
    pub(super) rebuild_supporting_files: bool,
}

pub(super) struct CollectedRoots {
    pub(super) files: Vec<PathBuf>,
    pub(super) inputs: Vec<PathBuf>,
    pub(super) reported: FxHashSet<PathBuf>,
    pub(super) package_routes: Vec<vize_canon::PackageRouteBinding>,
}

pub(super) fn split_program_candidates(
    collected: CollectedRoots,
    tsconfig_path: Option<&Path>,
    prefer_source_tsconfig: bool,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
) -> Vec<ProgramCandidate> {
    let CollectedRoots {
        files,
        inputs,
        reported,
        package_routes,
    } = collected;
    if prefer_source_tsconfig {
        let mut by_shell: BTreeMap<Option<PathBuf>, Vec<PathBuf>> = BTreeMap::new();
        for file in &inputs {
            let nearest = find_nearest_tsconfig_dir(file).map(|dir| dir.join("tsconfig.json"));
            by_shell.entry(nearest).or_default().push(file.clone());
        }
        let invocation = tsconfig_path.map(Path::to_path_buf);
        if by_shell.len() > 1 || by_shell.keys().next() != Some(&invocation) {
            let mut by_project: BTreeMap<Option<PathBuf>, Vec<PathBuf>> = BTreeMap::new();
            for (shell, source_files) in by_shell {
                let groups = resolve_tsconfig_program_inputs(
                    shell.as_deref(),
                    &source_files,
                    include_jsx,
                    cache,
                );
                if groups.is_empty() {
                    by_project.entry(shell).or_default().extend(source_files);
                } else {
                    for group in groups {
                        by_project
                            .entry(Some(group.tsconfig_path))
                            .or_default()
                            .extend(group.files);
                    }
                }
            }
            return by_project
                .into_iter()
                .map(|(tsconfig_path, files)| ProgramCandidate {
                    reported: canonical_file_set(&files, canonical_paths),
                    inputs: files.clone(),
                    files,
                    package_routes: Vec::new(),
                    tsconfig_path,
                    rebuild_supporting_files: true,
                })
                .collect();
        }
    }
    let groups = resolve_tsconfig_program_inputs(tsconfig_path, &inputs, include_jsx, cache);
    let single_group_uses_invocation_config = groups.first().is_none_or(|group| {
        tsconfig_path.is_none_or(|path| {
            canonical_paths.canonicalize(path) == canonical_paths.canonicalize(&group.tsconfig_path)
        })
    });
    if groups.len() <= 1 && single_group_uses_invocation_config {
        return vec![ProgramCandidate {
            files,
            inputs,
            reported,
            package_routes,
            tsconfig_path: groups
                .first()
                .map(|group| group.tsconfig_path.clone())
                .or_else(|| tsconfig_path.map(Path::to_path_buf)),
            rebuild_supporting_files: false,
        }];
    }

    // Each referenced program rebuilds its own reachable graph below; routes
    // collected from the solution shell must not leak across program scopes.
    drop(package_routes);
    groups
        .into_iter()
        .map(|group| {
            let reported = canonical_file_set(&group.files, canonical_paths);
            ProgramCandidate {
                inputs: group.files.clone(),
                files: group.files,
                reported,
                package_routes: Vec::new(),
                tsconfig_path: Some(group.tsconfig_path),
                rebuild_supporting_files: true,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_root_inputs_split_at_nearest_package_configs() {
        let root = tempfile::tempdir().unwrap();
        let root_config = root.path().join("tsconfig.json");
        std::fs::write(&root_config, r#"{"include":["*.ts"]}"#).unwrap();
        let mut files = Vec::new();
        for package in ["web", "admin"] {
            let dir = root.path().join("packages").join(package);
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::write(dir.join("tsconfig.json"), r#"{"include":["src/**/*"]}"#).unwrap();
            let file = dir.join("src/App.vue");
            std::fs::write(&file, "<template />").unwrap();
            files.push(file);
        }
        let collected = CollectedRoots {
            reported: FxHashSet::default(),
            package_routes: Vec::new(),
            inputs: files.clone(),
            files,
        };
        let mut cache = TsconfigInputCache::default();
        let mut paths = CanonicalPathCache::default();
        let candidates = split_program_candidates(
            collected,
            Some(&root_config),
            true,
            false,
            &mut cache,
            &mut paths,
        );
        assert_eq!(candidates.len(), 2);
        for candidate in candidates {
            assert_eq!(candidate.inputs.len(), 1);
            assert_eq!(
                candidate.tsconfig_path,
                candidate.inputs[0]
                    .parent()
                    .unwrap()
                    .parent()
                    .map(|dir| dir.join("tsconfig.json"))
            );
        }
    }
}

//! Splitting one check invocation into effective TypeScript programs.

use std::path::{Path, PathBuf};

use vize_s0::FxHashSet;

use super::default_imports::canonical_file_set;
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

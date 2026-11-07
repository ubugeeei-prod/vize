//! Splitting one check invocation into effective TypeScript programs.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use vize_l0::FxHashSet;

use super::default_imports::canonical_file_set;
use super::resolve::{find_nearest_tsconfig_path, find_nearest_tsconfig_path_within};
use crate::commands::check::{
    path_cache::CanonicalPathCache,
    tsconfig_inputs::{
        TsconfigInputCache, TsconfigProgramInputs, referenced_projects,
        resolve_tsconfig_program_inputs,
    },
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
    tsconfig_boundary: Option<&Path>,
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
            let nearest = tsconfig_boundary.map_or_else(
                || find_nearest_tsconfig_path(file),
                |boundary| find_nearest_tsconfig_path_within(file, boundary),
            );
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
    if keeps_invocation_program(&groups, tsconfig_path, canonical_paths) {
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

/// Order programs so every referenced project comes before its importers,
/// keeping the collection order among independent programs, and return the
/// configs other programs reference. A reference cycle keeps its collection
/// order; its members share nothing with each other.
pub(super) fn order_by_references(
    candidates: Vec<ProgramCandidate>,
    cache: &mut TsconfigInputCache,
) -> (Vec<ProgramCandidate>, FxHashSet<PathBuf>) {
    if candidates.len() < 2 {
        return (candidates, FxHashSet::default());
    }
    let configs: Vec<Option<PathBuf>> = candidates
        .iter()
        .map(|candidate| candidate.tsconfig_path.clone())
        .collect();
    let known: FxHashSet<&Path> = configs.iter().flatten().map(PathBuf::as_path).collect();
    let dependencies: Vec<Vec<PathBuf>> = configs
        .iter()
        .map(|config| {
            config.as_deref().map_or_else(Vec::new, |config| {
                referenced_projects(config, cache)
                    .into_iter()
                    .filter(|referenced| known.contains(referenced.as_path()))
                    .collect()
            })
        })
        .collect();
    let referenced: FxHashSet<PathBuf> = dependencies.iter().flatten().cloned().collect();
    let mut placed = Vec::with_capacity(candidates.len());
    let mut done: FxHashSet<PathBuf> = FxHashSet::default();
    let mut remaining: Vec<usize> = (0..candidates.len()).collect();
    loop {
        let before = placed.len();
        remaining.retain(|&index| {
            let ready = dependencies.get(index).is_some_and(|dependencies| {
                dependencies
                    .iter()
                    .all(|dependency| done.contains(dependency))
            });
            if ready {
                placed.push(index);
                if let Some(config) = configs.get(index).and_then(Option::as_ref) {
                    done.insert(config.clone());
                }
            }
            !ready
        });
        if placed.len() == before {
            break;
        }
    }
    placed.extend(remaining);
    let mut slots: Vec<Option<ProgramCandidate>> = candidates.into_iter().map(Some).collect();
    let ordered = placed
        .into_iter()
        .filter_map(|index| slots.get_mut(index).and_then(Option::take))
        .collect();
    (ordered, referenced)
}

/// Whether the invocation config stays the only program. Otherwise each
/// referenced program rebuilds its own closure from its inputs.
pub(super) fn keeps_invocation_program(
    groups: &[TsconfigProgramInputs],
    tsconfig_path: Option<&Path>,
    canonical_paths: &mut CanonicalPathCache,
) -> bool {
    groups.len() <= 1
        && groups.first().is_none_or(|group| {
            tsconfig_path.is_none_or(|path| {
                canonical_paths.canonicalize(path)
                    == canonical_paths.canonicalize(&group.tsconfig_path)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod jsconfig;

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
            None,
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

    fn candidate(tsconfig_path: Option<PathBuf>) -> ProgramCandidate {
        ProgramCandidate {
            files: Vec::new(),
            inputs: Vec::new(),
            reported: FxHashSet::default(),
            package_routes: Vec::new(),
            tsconfig_path,
            rebuild_supporting_files: true,
        }
    }

    /// `app` and `feat` import `kit`; `feat` also imports `app`. Whatever the
    /// collection order, each referenced project precedes its importers, and
    /// only the referenced ones emit declarations.
    #[test]
    fn referenced_programs_precede_their_importers() {
        let root = tempfile::tempdir().unwrap();
        let write = |relative: &str, content: &str| {
            let path = root.path().join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        };
        write("kit/tsconfig.json", r#"{"include":["src"]}"#);
        write(
            "app/tsconfig.json",
            r#"{"include":["src"],"references":[{"path":"../kit"}]}"#,
        );
        write(
            "feat/tsconfig.json",
            r#"{"include":["src"],"references":[{"path":"../kit"},{"path":"../app"}]}"#,
        );
        write("lone/tsconfig.json", r#"{"include":["src"]}"#);
        let config = |name: &str| {
            crate::commands::check::tsconfig_inputs::normalize_input_path(
                &root.path().join(name).join("tsconfig.json"),
            )
        };
        let mut cache = TsconfigInputCache::default();
        let (ordered, referenced) = order_by_references(
            vec![
                candidate(Some(config("feat"))),
                candidate(Some(config("lone"))),
                candidate(Some(config("app"))),
                candidate(None),
                candidate(Some(config("kit"))),
            ],
            &mut cache,
        );
        let order: Vec<_> = ordered
            .iter()
            .map(|candidate| {
                candidate
                    .tsconfig_path
                    .as_deref()
                    .and_then(|path| path.parent())
                    .and_then(|dir| dir.file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .collect();
        assert_eq!(order, ["lone", "", "kit", "app", "feat"]);
        assert_eq!(
            referenced,
            FxHashSet::from_iter([config("kit"), config("app")])
        );
        let _ = root.close();
    }

    /// A reference cycle keeps its collection order instead of hanging.
    #[test]
    fn reference_cycles_keep_collection_order() {
        let root = tempfile::tempdir().unwrap();
        for (name, other) in [("a", "b"), ("b", "a")] {
            let dir = root.path().join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("tsconfig.json"),
                format!(r#"{{"include":["src"],"references":[{{"path":"../{other}"}}]}}"#),
            )
            .unwrap();
        }
        let config = |name: &str| {
            crate::commands::check::tsconfig_inputs::normalize_input_path(
                &root.path().join(name).join("tsconfig.json"),
            )
        };
        let mut cache = TsconfigInputCache::default();
        let (ordered, referenced) = order_by_references(
            vec![candidate(Some(config("b"))), candidate(Some(config("a")))],
            &mut cache,
        );
        let order: Vec<_> = ordered
            .iter()
            .map(|candidate| candidate.tsconfig_path.clone().unwrap())
            .collect();
        assert_eq!(order, [config("b"), config("a")]);
        assert_eq!(referenced.len(), 2);
    }
}

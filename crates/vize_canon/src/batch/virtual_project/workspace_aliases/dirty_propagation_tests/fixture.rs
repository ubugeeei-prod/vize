//! Controlled prepared workspace graph, not the reporter's unavailable Nuxt project.

use std::{fs, path::PathBuf};

use vize_carton::{FxHashMap, FxHashSet, String};

use crate::{PackageResolutionContext, PackageRoute, PackageRouteBinding, PackageRouteSource};

use super::{PackageResolutionMode, PackageRouteKey, VirtualProject, reset, work};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/cold-package-shadow-routes-11265/input.json"
));

struct Graph {
    _directory: Option<tempfile::TempDir>,
    root: PathBuf,
    importers: Vec<PathBuf>,
    sources: Vec<PathBuf>,
    route: PackageRoute,
}

impl Graph {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
        Self::at(root, Some(directory))
    }

    fn at(root: PathBuf, directory: Option<tempfile::TempDir>) -> Self {
        let input: serde_json::Value = serde_json::from_str(INPUT).unwrap();
        assert_eq!(
            input["id"],
            "typechecker/regression/cold-package-shadow-routes-11265"
        );
        for file in input["files"].as_array().unwrap() {
            let path = root.join(file["path"].as_str().unwrap());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, file["source"].as_str().unwrap()).unwrap();
        }
        let config: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(".nuxt/tsconfig.app.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            config["compilerOptions"]["paths"]
                .as_object()
                .unwrap()
                .len(),
            39
        );
        let package = root.join("packages/kit");
        let stored = root.join("node_modules/.pnpm/@fixture+kit@1/node_modules/@fixture/kit");
        fs::create_dir_all(stored.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&package, &stored).unwrap();
        let installed = root.join("node_modules/@fixture/kit");
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&stored, &installed).unwrap();
        assert_eq!(fs::canonicalize(&installed).unwrap(), package);
        let sources = input["packageSources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|path| root.join(path.as_str().unwrap()))
            .collect::<Vec<_>>();
        let importers = input["importers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|path| root.join(path.as_str().unwrap()))
            .collect();
        let route = PackageRoute {
            source_paths: sources.clone(),
            dependency_paths: Vec::new(),
            source_targets: sources
                .iter()
                .map(|source| PackageRouteSource {
                    target_path: source.clone(),
                    source_path: source.clone(),
                    native_probe_path: if source.extension().is_some_and(|ext| ext == "vue") {
                        source.with_extension("d.vue.ts")
                    } else {
                        source.clone()
                    },
                })
                .collect(),
            package_root: package.clone(),
            package_link_root: installed,
            manifest_path: package.join("package.json"),
            package_name: Some("@fixture/kit".into()),
            workspace_source: true,
            nested_routes: Vec::new(),
        };
        Self {
            _directory: directory,
            root,
            importers,
            sources,
            route,
        }
    }

    fn registered(&self) -> VirtualProject {
        let mut project = VirtualProject::new(&self.root).unwrap();
        project.set_tsconfig_path(Some(self.root.join(".nuxt/tsconfig.app.json")));
        project.set_package_routes(self.importers.iter().map(|importer| PackageRouteBinding {
            importer_path: importer.clone(),
            specifier: "@fixture/kit".into(),
            occurrence_mode: PackageResolutionMode::Contextual,
            context: PackageResolutionContext::default(),
            route: Some(self.route.clone()),
            invalidation_paths: self.route.invalidation_paths(),
            shared_invalidation_paths: Vec::new(),
        }));
        project.register_paths(&self.importers[..2]).unwrap();
        assert!(!project.package_shadows_initialized);
        reset();
        project.register_package_route_targets().unwrap();
        assert!(!project.package_shadows_initialized);
        for source in &self.sources {
            assert!(project.find_by_original(source).is_some(), "{source:?}");
        }
        assert!(
            project
                .package_route_roots
                .values()
                .any(|owners| owners.len() >= self.importers.len())
        );
        project
    }
}

/// Whole source bytes, import maps, SFC projection/metadata, and coarse block
/// classifications: no offset translation or packet normalization.
#[derive(Debug, PartialEq, Eq)]
struct File {
    original: PathBuf,
    generated: PathBuf,
    code: String,
    imports: crate::batch::import_rewriter::ImportSourceMap,
    projection: Option<crate::virtual_ts::ProjectionMapping>,
    blocks: Vec<crate::batch::SfcBlockType>,
}

#[derive(Debug, PartialEq, Eq)]
struct Shadow {
    files: FxHashMap<PathBuf, PathBuf>,
    manifests: FxHashMap<PathBuf, PathBuf>,
    aliases: FxHashMap<PathBuf, PathBuf>,
    install_roots: FxHashMap<PathBuf, PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    routes: Vec<PackageRouteBinding>,
    files: Vec<File>,
    originals: FxHashMap<PathBuf, String>,
    shadows: FxHashMap<PathBuf, (PathBuf, String)>,
    manifests: FxHashMap<PathBuf, (PathBuf, Vec<u8>)>,
    file_owners: FxHashMap<PathBuf, super::super::super::PackageShadowOwners>,
    manifest_owners: FxHashMap<PathBuf, super::super::super::PackageShadowOwners>,
    source_index: FxHashMap<PathBuf, FxHashMap<PathBuf, (PathBuf, PathBuf)>>,
    source_owners: FxHashMap<PathBuf, usize>,
    source_paths: FxHashMap<PathBuf, usize>,
    root_owners: FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    importer_owners: FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    manifest_routes: FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    shadow_sources: FxHashMap<PathBuf, FxHashSet<PathBuf>>,
    shadow_scopes: FxHashMap<String, Vec<PathBuf>>,
    artifacts: FxHashMap<PackageRouteKey, Shadow>,
    reverse: FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    aliases: FxHashMap<PathBuf, PathBuf>,
}

fn snapshot(project: &VirtualProject, graph: &Graph) -> Snapshot {
    Snapshot {
        routes: project.package_routes_snapshot(),
        files: project
            .virtual_files_sorted()
            .into_iter()
            .map(|file| File {
                original: file.original_path.clone(),
                generated: file.virtual_path.clone(),
                code: file.content.clone(),
                imports: file.source_map.import_map.clone(),
                projection: file
                    .source_map
                    .sfc_map
                    .as_ref()
                    .map(|map| map.projection().clone()),
                blocks: file
                    .source_map
                    .sfc_map
                    .as_ref()
                    .map(|map| {
                        let source = &project.original_contents[&file.virtual_path];
                        (0..=source.len())
                            .map(|offset| map.block_type_at(offset as u32))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect(),
        originals: project.original_contents.clone(),
        shadows: project
            .package_shadow_files
            .iter()
            .map(|(path, owner)| {
                (
                    path.clone(),
                    (
                        owner.clone(),
                        project.package_shadow_content(path, owner).unwrap(),
                    ),
                )
            })
            .collect(),
        manifests: project
            .package_shadow_manifests
            .iter()
            .map(|(path, owner)| (path.clone(), (owner.clone(), fs::read(owner).unwrap())))
            .collect(),
        file_owners: project.package_shadow_file_owners.clone(),
        manifest_owners: project.package_shadow_manifest_owners.clone(),
        source_index: project.package_source_index.clone(),
        source_owners: project.package_route_source_owners.clone(),
        source_paths: project.package_route_source_paths.clone(),
        root_owners: project.package_route_roots.clone(),
        importer_owners: project.package_route_importers.clone(),
        manifest_routes: project.package_route_manifests.clone(),
        shadow_sources: project.package_shadow_source_paths.clone(),
        shadow_scopes: project.package_shadow_scopes.clone(),
        artifacts: project
            .package_shadow_artifacts
            .iter()
            .map(|(key, topology)| {
                (
                    key.clone(),
                    Shadow {
                        files: topology.files.clone(),
                        manifests: topology.manifests.clone(),
                        aliases: topology.aliases.clone(),
                        install_roots: topology.install_roots.clone(),
                    },
                )
            })
            .collect(),
        reverse: graph
            .route
            .invalidation_paths()
            .into_iter()
            .chain(graph.importers.iter().cloned())
            .map(|path| {
                let keys = project.package_route_keys_for_changes(std::slice::from_ref(&path));
                (path, keys)
            })
            .collect(),
        aliases: project
            .workspace_alias_links()
            .into_iter()
            .map(|link| (link.virtual_dir, link.real_dir))
            .collect(),
    }
}

#[test]
fn prepared_pnpm_cycles_keep_complete_cold_warm_rebind_and_owner_repair_state() {
    let graph = Graph::new();
    let mut project = graph.registered();
    assert!(work().cold_calls > 0);
    assert_eq!(work().pushed, 0);
    assert_eq!(work().popped, 0);
    history(&graph, &mut project, |_, _| {});
}

fn history(
    graph: &Graph,
    project: &mut VirtualProject,
    mut observe: impl FnMut(&str, &VirtualProject),
) {
    reset();
    // Removed owners before first initialization still have explicit dirty seeds.
    let removed = project
        .package_routes_snapshot()
        .into_iter()
        .find(|binding| binding.importer_path == graph.importers[0])
        .unwrap();
    let mut cold_dirty = project.package_shadow_dirty_keys.clone();
    cold_dirty.insert(removed.key());
    project
        .remove_package_route_binding(&removed.key())
        .unwrap();
    assert_eq!(project.package_shadow_dirty_keys, cold_dirty);
    project.insert_package_route_binding(removed.clone());
    project.finalize_package_routes().unwrap();
    assert!(project.package_shadows_initialized);
    assert!(project.package_shadow_dirty_keys.is_empty());
    assert!(!project.package_shadow_files.is_empty());
    let cold = snapshot(project, graph);
    observe("cold", project);
    reset();
    project.finalize_package_routes().unwrap();
    assert_eq!(snapshot(project, graph), cold);
    observe("warm-noop", project);

    // Warm rebind must retain all co-owners, full code/maps, reverse keys and aliases.
    reset();
    project.insert_package_route_binding(removed.clone());
    project.finalize_package_routes().unwrap();
    assert_eq!(snapshot(project, graph), cold);
    observe("warm-rebind", project);
    reset();
    project
        .remove_package_route_binding(&removed.key())
        .unwrap();
    project.finalize_package_routes().unwrap();
    let stale_owners = project
        .package_shadow_file_owners
        .iter()
        .filter(|(_, owners)| owners.contains_key(&removed.key()))
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    assert_eq!(stale_owners, Vec::<PathBuf>::new());
    assert!(
        project
            .package_shadow_file_owners
            .values()
            .any(|owners| !owners.is_empty())
    );
    observe("owner-removed", project);
    reset();
    project.insert_package_route_binding(removed);
    project.finalize_package_routes().unwrap();
    assert_eq!(snapshot(project, graph), cold);
    observe("owner-repair", project);

    reset();
    let bindings = project.package_routes_snapshot();
    for binding in &bindings {
        project
            .remove_package_route_binding(&binding.key())
            .unwrap();
    }
    project.finalize_package_routes().unwrap();
    assert!(project.package_shadow_files.is_empty());
    assert!(project.package_shadow_manifests.is_empty());
    assert!(project.package_shadow_file_owners.is_empty());
    assert!(project.package_shadow_manifest_owners.is_empty());
    observe("all-removed", project);
    reset();
    project.set_package_routes(bindings);
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();
    assert_eq!(snapshot(project, graph), cold);
    observe("all-repair", project);
}

mod capture;

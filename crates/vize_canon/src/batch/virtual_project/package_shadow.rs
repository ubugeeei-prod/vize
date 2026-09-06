//! Importer-scoped package topology inside Canon's virtual project.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, FxHashSet, cstr};

use crate::batch::error::CorsaResult;
use crate::package_route::PackageRouteKey;

use super::VirtualProject;
use super::build::mirrored_virtual_path;

#[derive(Clone, Default)]
pub(super) struct PackageShadowTopology {
    pub(super) files: FxHashMap<PathBuf, PathBuf>,
    pub(super) manifests: FxHashMap<PathBuf, PathBuf>,
}

impl VirtualProject {
    pub(super) fn rebuild_package_shadows(&mut self) -> CorsaResult<()> {
        if !self.package_shadows_initialized {
            self.package_shadow_files.clear();
            self.package_shadow_manifests.clear();
            self.package_shadow_artifacts.clear();
            self.package_shadow_file_owners.clear();
            self.package_shadow_manifest_owners.clear();
            self.package_shadow_source_paths.clear();
            self.package_shadow_dirty_keys
                .extend(self.package_routes.keys().cloned());
            self.package_shadows_initialized = true;
        }

        self.refresh_package_shadow_scopes();
        let mut dirty = std::mem::take(&mut self.package_shadow_dirty_keys)
            .into_iter()
            .collect::<Vec<_>>();
        dirty.sort();
        self.incremental_shadow_bindings_rebuilt += dirty.len();
        for key in dirty {
            self.refresh_package_shadow(&key)?;
        }
        Ok(())
    }

    fn refresh_package_shadow(&mut self, key: &PackageRouteKey) -> CorsaResult<()> {
        self.remove_package_shadow_owner(key);
        let Some(binding) = self.package_routes.get(key).cloned() else {
            return Ok(());
        };
        let Some(route) = binding.route.as_ref() else {
            return Ok(());
        };
        let mut topology = PackageShadowTopology::default();

        if let Some(scope_manifest) = binding.context.scope_manifest_path.as_ref()
            && scope_manifest.is_file()
            && let Ok(virtual_manifest) =
                mirrored_virtual_path(&self.project_root, &self.virtual_root, scope_manifest)
        {
            topology
                .manifests
                .insert(virtual_manifest, scope_manifest.clone());
        }

        // Declaration emit uses Canon's canonical materialized identities.
        // Give them the same native package boundary/probes as importer roots.
        if let Ok(canonical_root) =
            mirrored_virtual_path(&self.project_root, &self.virtual_root, &route.package_root)
        {
            self.collect_route_shadow_topology(
                route,
                &canonical_root,
                &mut FxHashSet::default(),
                &mut topology,
            );
        }

        if let Some(importer_virtual_path) = self
            .find_by_original(&binding.importer_path)
            .map(|file| file.virtual_path.clone())
        {
            let shadow_roots = if binding.specifier.starts_with('#') {
                vec![private_package_shadow_root(
                    &self.virtual_root,
                    &route.manifest_path,
                )]
            } else if let (Some(package_name), Some(importer_dir)) = (
                route.package_name.as_deref(),
                importer_virtual_path.parent(),
            ) {
                self.package_shadow_scope_dirs(package_name, importer_dir)
                    .into_iter()
                    .map(|scope| scope.join("node_modules").join(package_name))
                    .collect()
            } else {
                self.install_package_shadow_owner(key.clone(), topology);
                return Ok(());
            };
            for shadow_root in shadow_roots {
                self.collect_route_shadow_topology(
                    route,
                    &shadow_root,
                    &mut FxHashSet::default(),
                    &mut topology,
                );
            }
        }
        self.install_package_shadow_owner(key.clone(), topology);
        Ok(())
    }

    /// Recompute the shared scope map and dirty every binding whose package
    /// moved, so a scope that opens or closes as importers arrive or leave is
    /// reinstalled instead of leaving a stale copy in the program (#4153).
    fn refresh_package_shadow_scopes(&mut self) {
        let next = self.shared_package_shadow_scopes();
        let drifted = self.package_shadow_scope_drift(&next);
        self.package_shadow_scopes = next;
        if drifted.is_empty() {
            return;
        }
        let keys = self
            .package_routes
            .iter()
            .filter(|(_, binding)| {
                binding
                    .route
                    .as_ref()
                    .and_then(|route| route.package_name.as_deref())
                    .is_some_and(|name| drifted.contains(name))
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        self.package_shadow_dirty_keys.extend(keys);
    }

    fn private_dependencies_for(&self, manifest: &Path) -> Vec<crate::PackageRoute> {
        let mut routes = self
            .package_route_manifests
            .get(manifest)
            .into_iter()
            .flatten()
            .filter_map(|key| self.package_routes.get(key))
            .filter(|binding| binding.specifier.starts_with('#'))
            .filter_map(|binding| binding.route.as_ref())
            .flat_map(|route| route.nested_routes.iter().cloned())
            .collect::<Vec<_>>();
        routes.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
        routes.dedup();
        routes
    }

    fn collect_route_shadow_topology(
        &self,
        route: &crate::PackageRoute,
        shadow_root: &Path,
        ancestors: &mut FxHashSet<PathBuf>,
        topology: &mut PackageShadowTopology,
    ) {
        if !ancestors.insert(route.manifest_path.clone()) {
            return;
        }
        topology.manifests.insert(
            shadow_root.join("package.json"),
            route.manifest_path.clone(),
        );
        self.collect_package_source_shadows(&route.package_root, shadow_root, topology);
        self.collect_manifest_target_shadows(route, shadow_root, topology);
        let mut nested_routes = route.nested_routes.iter().collect::<Vec<_>>();
        let private_dependencies = self.private_dependencies_for(&route.manifest_path);
        nested_routes.extend(private_dependencies.iter());
        nested_routes.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
        nested_routes.dedup();
        for nested in nested_routes {
            let Some(package_name) = nested.package_name.as_deref() else {
                continue;
            };
            self.collect_route_shadow_topology(
                nested,
                &shadow_root.join("node_modules").join(package_name),
                ancestors,
                topology,
            );
        }
        ancestors.remove(&route.manifest_path);
    }

    fn collect_manifest_target_shadows(
        &self,
        route: &crate::PackageRoute,
        shadow_root: &Path,
        topology: &mut PackageShadowTopology,
    ) {
        for source in &route.source_targets {
            let Some(canonical_virtual) = self
                .find_by_original(&source.source_path)
                .map(|file| file.virtual_path.clone())
            else {
                continue;
            };
            let Some(relative_probe) = source.native_probe_relative_path(&route.package_root)
            else {
                continue;
            };
            topology
                .files
                .entry(shadow_root.join(relative_probe))
                .or_insert_with(|| canonical_virtual.clone());
            if let Some(relative_target) =
                source.declaration_target_relative_path(&route.package_root)
            {
                topology
                    .files
                    .entry(shadow_root.join(relative_target))
                    .or_insert_with(|| canonical_virtual.clone());
            }
            self.collect_workspace_dependency_target_shadows(route, source, shadow_root, topology);
        }
    }

    fn collect_workspace_dependency_target_shadows(
        &self,
        route: &crate::PackageRoute,
        source: &crate::PackageRouteSource,
        shadow_root: &Path,
        topology: &mut PackageShadowTopology,
    ) {
        let normalized_root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
        let Some(indexed_sources) = self.package_source_index.get(&normalized_root) else {
            return;
        };
        let mut sources = indexed_sources
            .iter()
            .map(|(source_path, (_, virtual_path))| (source_path, virtual_path))
            .collect::<Vec<_>>();
        sources.sort_by(|left, right| left.0.cmp(right.0));

        for (dependency, canonical_virtual) in sources {
            let Some(relative_probe) =
                source.workspace_dependency_probe_relative_path(&route.package_root, dependency)
            else {
                continue;
            };
            topology
                .files
                .entry(shadow_root.join(relative_probe))
                .or_insert_with(|| canonical_virtual.clone());
        }
    }

    fn collect_package_source_shadows(
        &self,
        source_root: &Path,
        shadow_root: &Path,
        topology: &mut PackageShadowTopology,
    ) {
        let normalized_root = vize_carton::path::canonicalize_non_verbatim(source_root);
        let mut source_files = self
            .package_source_index
            .get(&normalized_root)
            .into_iter()
            .flatten()
            .map(|(_, (relative, virtual_path))| (relative.clone(), virtual_path.clone()))
            .collect::<Vec<_>>();
        source_files.sort_by(|left, right| left.0.cmp(&right.0));

        for (relative, canonical_virtual) in source_files {
            let destination = shadow_root.join(&relative);
            if relative
                .extension()
                .is_some_and(|extension| extension == "vue")
            {
                let Some(name) = destination.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                let suffix = if canonical_virtual
                    .extension()
                    .is_some_and(|extension| extension == "tsx")
                {
                    ".tsx"
                } else {
                    ".ts"
                };
                topology
                    .files
                    .entry(destination.with_file_name(cstr!("{name}{suffix}")))
                    .or_insert_with(|| canonical_virtual.clone());
                let stem = relative
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(name);
                topology
                    .files
                    .entry(destination.with_file_name(cstr!("{stem}.d.vue.ts")))
                    .or_insert(canonical_virtual);
            } else {
                topology.files.insert(destination, canonical_virtual);
            }
        }
    }
}

fn private_package_shadow_root(virtual_root: &Path, manifest_path: &Path) -> PathBuf {
    let manifest_path = vize_carton::path::canonicalize_non_verbatim(manifest_path);
    let mut digest = Sha256::new();
    digest.update(b"vize-private-package-shadow:v1\0");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        digest.update(manifest_path.as_os_str().as_bytes());
    }
    #[cfg(not(unix))]
    digest.update(manifest_path.as_os_str().to_string_lossy().as_bytes());
    let mut key = vize_carton::String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest.finalize() {
        key.push(HEX[(byte >> 4) as usize] as char);
        key.push(HEX[(byte & 0x0f) as usize] as char);
    }
    virtual_root
        .join("__vize_package_scopes__")
        .join(key.as_str())
}

//! Preserve real in-package install scopes inside workspace shadow copies.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::{PackageShadowTopology, VirtualProject};

type PackageIdentity = (PathBuf, PathBuf);

pub(super) struct WorkspaceShadowPlan {
    incoming: FxHashMap<PathBuf, FxHashSet<PackageIdentity>>,
}

fn identity(route: &crate::PackageRoute) -> PackageIdentity {
    (
        vize_carton::path::canonicalize_non_verbatim(&route.package_root),
        vize_carton::path::canonicalize_non_verbatim(&route.manifest_path),
    )
}

impl WorkspaceShadowPlan {
    fn target(
        &self,
        importer: &Path,
        name: &str,
        route: &crate::PackageRoute,
        alias: &Path,
    ) -> Option<PathBuf> {
        let expected = identity(route);
        for directory in importer.parent()?.ancestors() {
            let candidate = directory.join("node_modules").join(name);
            let Some(identities) = self.incoming.get(&candidate) else {
                continue;
            };
            // The nearest visible root is authoritative, including a conflict.
            // A farther same-name package must not hide a divergent install.
            if identities.len() != 1
                || !identities.contains(&expected)
                || candidate.starts_with(alias)
                || alias.starts_with(&candidate)
            {
                return None;
            }
            return Some(candidate);
        }
        None
    }
}

impl VirtualProject {
    pub(super) fn workspace_shadow_plan(&self) -> WorkspaceShadowPlan {
        let mut incoming = FxHashMap::<_, FxHashSet<_>>::default();
        for binding in self.package_routes.values() {
            if binding.specifier.starts_with('#') {
                continue;
            }
            let Some(route) = binding.route.as_ref() else {
                continue;
            };
            let Some(name) = route.package_name.as_deref() else {
                continue;
            };
            let Some(directory) = self
                .find_by_original(&binding.importer_path)
                .and_then(|file| file.virtual_path.parent())
            else {
                continue;
            };
            for scope in self.package_shadow_scope_dirs(name, directory) {
                incoming
                    .entry(scope.join("node_modules").join(name))
                    .or_default()
                    .insert(identity(route));
            }
        }
        WorkspaceShadowPlan { incoming }
    }

    pub(super) fn private_dependencies_for(&self, manifest: &Path) -> Vec<crate::PackageRoute> {
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

    pub(super) fn collect_workspace_dependency_shadows(
        &self,
        route: &crate::PackageRoute,
        shadow_root: &Path,
        ancestors: &mut FxHashSet<PathBuf>,
        topology: &mut PackageShadowTopology,
        plan: &WorkspaceShadowPlan,
    ) {
        if !route.workspace_source {
            return;
        }
        let root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
        let Some(sources) = self.package_source_index.get(&root) else {
            return;
        };
        let mut dependencies = Vec::new();
        for (source, (source_relative, _)) in sources {
            let Some(keys) = self.package_route_importers.get(source) else {
                continue;
            };
            for key in keys {
                let Some(binding) = self.package_routes.get(key) else {
                    continue;
                };
                // Private imports already retain their manifest-keyed topology.
                if binding.specifier.starts_with('#') {
                    continue;
                }
                let Some(dependency) = binding.route.as_ref() else {
                    continue;
                };
                let Some(name) = dependency.package_name.as_deref() else {
                    continue;
                };
                let Ok(relative) = dependency.package_link_root.strip_prefix(&root) else {
                    // A root/shared install already uses the common shadow
                    // scope. Inventing deeper copies splits class identity.
                    continue;
                };
                if !relative.ends_with(Path::new("node_modules").join(name)) {
                    continue;
                }
                let shadow = shadow_root.join(relative);
                let target = dependency
                    .workspace_source
                    .then(|| {
                        plan.target(
                            &shadow_root.join(source_relative),
                            name,
                            dependency,
                            &shadow,
                        )
                    })
                    .flatten();
                dependencies.push((shadow, dependency, target));
            }
        }
        dependencies.sort_by(|(left, left_route, _), (right, right_route, _)| {
            (left, &left_route.manifest_path).cmp(&(right, &right_route.manifest_path))
        });
        dependencies.dedup();
        let mut start = 0;
        while start < dependencies.len() {
            let (shadow, dependency, target) = &dependencies[start];
            let end = start + dependencies[start..].partition_point(|entry| entry.0 == *shadow);
            let shared_target = target.as_ref().filter(|target| {
                dependencies[start..end]
                    .iter()
                    .all(|(_, route, candidate)| {
                        identity(route) == identity(dependency)
                            && candidate.as_ref() == Some(*target)
                    })
            });
            if let Some(target) = shared_target {
                topology.aliases.insert(shadow.clone(), target.clone());
                // Pin the target's authored files and raw manifest under this
                // owner too; another binding's refresh/removal cannot dangle it.
                self.collect_route_shadow_topology(dependency, target, ancestors, topology, plan);
            } else {
                // An uncertain or divergent incoming scope retains the bound
                // installation's own topology rather than aliasing by name.
                for (_, route, _) in &dependencies[start..end] {
                    self.collect_route_shadow_topology(route, shadow, ancestors, topology, plan);
                }
            }
            start = end;
        }
    }
}

//! Identity-scoped placement for materialized package shadows (#4153).
//!
//! A bare specifier keeps its importer-scoped identity (#4002/#4137): the route
//! that resolved it is the authority for which physical package the importer
//! sees. The *materialized* shadow, though, only has to be importer-scoped
//! where two importers disagree. Anchoring every scope at
//! `<importer dir>/node_modules/<name>` copies each package source once per
//! importing directory, so a workspace whose packages are imported from
//! hundreds of directories materializes — and type-checks, because
//! [`super::tsconfig_gen`] lists shadow copies of declaration roots as program
//! roots — the same authored file hundreds of times.
//!
//! When every occurrence of a package name in the project resolved to the same
//! physical package, one scope at the deepest common ancestor of those
//! importers serves all of them unchanged: Node's ancestor walk stops at the
//! nearest `node_modules/<name>`, so an importer-scoped copy still wins
//! wherever identities do diverge, and a name with more than one identity keeps
//! per-importer scopes exactly as before. A single importing directory folds to
//! itself, so projects without fan-out materialize byte-identical trees.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use super::VirtualProject;

/// The physical package one bare specifier resolved to: canonical package root
/// and the manifest that selected it. Two routes agreeing on both name the same
/// installed package even when they were reached through different links.
type ShadowIdentity = (PathBuf, PathBuf);

impl VirtualProject {
    /// Shared scope directories per package name, for names whose every
    /// resolved route in this project is the same physical package.
    pub(super) fn shared_package_shadow_scopes(&self) -> FxHashMap<CompactString, Vec<PathBuf>> {
        let mut identities: FxHashMap<CompactString, FxHashSet<ShadowIdentity>> =
            FxHashMap::default();
        let mut importer_dirs: FxHashMap<CompactString, Vec<PathBuf>> = FxHashMap::default();
        for binding in self.package_routes.values() {
            // A private `#` import already resolves through a manifest-keyed
            // scope, which is shared by construction.
            if binding.specifier.starts_with('#') {
                continue;
            }
            let Some(route) = binding.route.as_ref() else {
                continue;
            };
            let Some(package_name) = route.package_name.as_deref() else {
                continue;
            };
            let Some(importer_dir) = self
                .find_by_original(&binding.importer_path)
                .and_then(|file| file.virtual_path.parent().map(Path::to_path_buf))
            else {
                continue;
            };
            let package_name = CompactString::from(package_name);
            identities
                .entry(package_name.clone())
                .or_default()
                .insert(shadow_identity(route));
            importer_dirs
                .entry(package_name)
                .or_default()
                .push(importer_dir);
        }

        let hoistable = identities
            .into_iter()
            .filter(|(_, identities)| identities.len() == 1)
            .filter_map(|(package_name, _)| {
                let dirs = importer_dirs.remove(&package_name)?;
                let scopes = self.hoisted_scope_dirs(&package_name, &dirs, 0);
                (!scopes.is_empty()).then_some((package_name, scopes))
            })
            .collect::<Vec<_>>();
        // Every hoisted name is installed at *every* shared scope directory.
        // A package's own sources resolve their siblings from inside the scope
        // they were materialized into, so a scope that hosts one package but
        // not the others it imports would move a resolution edge out of the
        // mirror. Sharing the directory set keeps each scope self-contained.
        let mut shared = hoistable
            .iter()
            .flat_map(|(_, scopes)| scopes.iter().cloned())
            .collect::<Vec<_>>();
        shared.sort();
        shared.dedup();
        hoistable
            .into_iter()
            .map(|(package_name, _)| (package_name, shared.clone()))
            .collect()
    }

    /// The shallowest collision-free directories that still dominate every
    /// importing directory.
    ///
    /// A mirror directory whose real counterpart owns a `node_modules` tree is
    /// refused: [`super::package_node_modules`] bridges exactly those into the
    /// real install, and a shadow written under one would let native resolution
    /// leave the mirror. Such an ancestor is split by its next path component
    /// instead, which still folds hundreds of importing directories into a
    /// handful of scopes without inventing a new resolution edge.
    fn hoisted_scope_dirs(
        &self,
        package_name: &str,
        dirs: &[PathBuf],
        depth: usize,
    ) -> Vec<PathBuf> {
        const MAX_SPLIT_DEPTH: usize = 32;
        let Some(common) = shared_scope_dir(dirs, &self.virtual_root) else {
            return dirs.to_vec();
        };
        if dirs.iter().all(|dir| *dir == common) {
            return vec![common];
        }
        let refused = self.scope_dir_reaches_real_install(&common)
            || (common == self.virtual_root && is_canon_runtime_entry(package_name));
        if depth >= MAX_SPLIT_DEPTH || !refused {
            return vec![common];
        }
        let mut groups: FxHashMap<PathBuf, Vec<PathBuf>> = FxHashMap::default();
        for dir in dirs {
            let child = dir
                .strip_prefix(&common)
                .ok()
                .and_then(|relative| relative.components().next())
                .map_or_else(|| dir.clone(), |component| common.join(component));
            groups.entry(child).or_default().push(dir.clone());
        }
        let mut scopes = groups
            .into_values()
            .flat_map(|group| self.hoisted_scope_dirs(package_name, &group, depth + 1))
            .collect::<Vec<_>>();
        scopes.sort();
        scopes.dedup();
        scopes
    }

    /// Whether a mirrored directory's real counterpart owns a `node_modules`
    /// tree, which the package-link pass mirrors into the virtual project.
    fn scope_dir_reaches_real_install(&self, scope: &Path) -> bool {
        let real = match scope.strip_prefix(&self.virtual_root) {
            Ok(relative) => self.project_root.join(relative),
            Err(_) => return true,
        };
        let real = super::external_mirror::external_mirror_original_path(scope).unwrap_or(real);
        real.join("node_modules").exists()
    }

    /// The directories whose `node_modules/<name>` host this importer's shadow.
    ///
    /// Falls back to the importer's own directory whenever the name has no
    /// single project-wide identity, so divergent installs keep the deeper —
    /// and therefore winning — scope they already had.
    pub(super) fn package_shadow_scope_dirs<'a>(
        &'a self,
        package_name: &str,
        importer_dir: &'a Path,
    ) -> Vec<&'a Path> {
        let scopes = self
            .package_shadow_scopes
            .get(package_name)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if scopes.is_empty() {
            return vec![importer_dir];
        }
        let mut dirs = scopes.iter().map(PathBuf::as_path).collect::<Vec<_>>();
        // An importer outside every shared scope keeps its own, so no occurrence
        // ever loses the shadow it resolved through.
        if !dirs.iter().any(|scope| importer_dir.starts_with(scope)) {
            dirs.push(importer_dir);
        }
        dirs
    }

    /// Package names whose shared scope changed, so their bindings rebuild.
    pub(super) fn package_shadow_scope_drift(
        &self,
        next: &FxHashMap<CompactString, Vec<PathBuf>>,
    ) -> FxHashSet<CompactString> {
        let mut drifted = FxHashSet::default();
        for (name, scope) in next {
            if self.package_shadow_scopes.get(name) != Some(scope) {
                drifted.insert(name.clone());
            }
        }
        for name in self.package_shadow_scopes.keys() {
            if !next.contains_key(name) {
                drifted.insert(name.clone());
            }
        }
        drifted
    }

    /// Shared scope entries that sit directly inside the mirror's own
    /// `node_modules`, which the runtime-dependency pass would otherwise prune
    /// and rewrite on every check. The runtime prune only inspects the entries
    /// immediately below `node_modules`, so a scoped name is preserved through
    /// its `@scope` directory.
    pub(crate) fn root_package_shadow_scope_entries(&self) -> Vec<PathBuf> {
        let root_modules = self.virtual_root.join("node_modules");
        let mut entries = self
            .package_shadow_scopes
            .iter()
            .filter(|(_, scopes)| scopes.contains(&self.virtual_root))
            .map(|(name, _)| root_modules.join(name.split('/').next().unwrap_or(name.as_str())))
            .collect::<Vec<_>>();
        entries.sort();
        entries.dedup();
        entries
    }
}

/// Entries the runtime-dependency pass owns directly below the mirror root's
/// `node_modules`.
fn is_canon_runtime_entry(package_name: &str) -> bool {
    matches!(
        package_name.split('/').next().unwrap_or(package_name),
        "vue" | "@vue" | "vite"
    )
}

fn shadow_identity(route: &crate::PackageRoute) -> ShadowIdentity {
    (
        vize_carton::path::canonicalize_non_verbatim(&route.package_root),
        vize_carton::path::canonicalize_non_verbatim(&route.manifest_path),
    )
}

/// The deepest directory that is an ancestor of every importing directory, or
/// `None` when they do not share one inside the mirror.
fn shared_scope_dir(dirs: &[PathBuf], virtual_root: &Path) -> Option<PathBuf> {
    let mut common = dirs.first()?.clone();
    for dir in dirs.iter().skip(1) {
        while !dir.starts_with(&common) {
            if !common.pop() {
                return None;
            }
        }
    }
    common.starts_with(virtual_root).then_some(common)
}

#[cfg(test)]
#[path = "tests/package_shadow_fan_out.rs"]
mod fan_out_tests;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn one_importer_directory_keeps_its_own_scope() {
        let root = Path::new("/v");
        let dirs = vec![PathBuf::from("/v/apps/a/src")];
        assert_eq!(
            super::shared_scope_dir(&dirs, root),
            Some(PathBuf::from("/v/apps/a/src"))
        );
    }

    #[test]
    fn sibling_importers_fold_to_their_common_ancestor() {
        let root = Path::new("/v");
        let dirs = vec![
            PathBuf::from("/v/apps/a/src"),
            PathBuf::from("/v/apps/b/src/views"),
            PathBuf::from("/v/apps/c"),
        ];
        assert_eq!(
            super::shared_scope_dir(&dirs, root),
            Some(PathBuf::from("/v/apps"))
        );
    }

    #[test]
    fn importers_across_the_mirror_fold_to_the_virtual_root() {
        let root = Path::new("/v");
        let dirs = vec![
            PathBuf::from("/v/apps/a/src"),
            PathBuf::from("/v/packages/ui/src"),
        ];
        assert_eq!(
            super::shared_scope_dir(&dirs, root),
            Some(PathBuf::from("/v"))
        );
    }

    #[test]
    fn an_importer_outside_the_mirror_has_no_shared_scope() {
        let root = Path::new("/v");
        let dirs = vec![PathBuf::from("/v/apps/a"), PathBuf::from("/other/pkg")];
        assert_eq!(super::shared_scope_dir(&dirs, root), None);
    }
}

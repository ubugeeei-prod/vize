//! Runtime content and links for importer-scoped package shadows.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet, String as CompactString, cstr};

use crate::batch::declaration_path::is_declaration_file;
use crate::batch::error::CorsaResult;

use super::VirtualProject;
use super::package_node_modules::{
    MaterializedPrefixIndex, PackageNodeModulesLink, merge_aware_node_modules_links,
};

impl VirtualProject {
    pub(super) fn package_shadow_content(
        &self,
        shadow_path: &Path,
        canonical_path: &Path,
    ) -> CorsaResult<CompactString> {
        let file = self.virtual_files.get(canonical_path).ok_or_else(|| {
            crate::batch::error::CorsaError::PathError {
                path: canonical_path.to_path_buf(),
            }
        })?;
        if file
            .original_path
            .extension()
            .is_none_or(|extension| extension != "vue")
        {
            if !is_declaration_file(&file.original_path)
                && let Some(forwarder) = declaration_shadow_forwarder(shadow_path)
            {
                return Ok(forwarder);
            }
            // Package TS/declaration modules keep their authored spelling in
            // the importer-local mirror. Native TypeScript can then apply
            // `allowArbitraryExtensions` to `./Widget.vue` and select the
            // generated `.d.vue.ts` companion itself. Reusing the canonical
            // project rewrite here would turn that edge into
            // `./Widget.vue.ts`; standard tsgo accepts it in batch mode, but
            // its editor project does not retain the dependency's typed
            // surface through a re-export barrel. The authored bytes are also
            // the only sound authority for import/export mode and diagnostics.
            return Ok(self
                .original_contents
                .get(canonical_path)
                .cloned()
                .unwrap_or_else(|| file.content.clone()));
        }
        if shadow_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".d.vue.ts"))
        {
            let source_name = file
                .original_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| crate::batch::error::CorsaError::PathError {
                    path: file.original_path.clone(),
                })?;
            let suffix = if file
                .virtual_path
                .extension()
                .is_some_and(|extension| extension == "tsx")
            {
                ".tsx"
            } else {
                ".ts"
            };
            return Ok(cstr!(
                "export {{ default }} from \"./{source_name}{suffix}\";\nexport * from \"./{source_name}{suffix}\";\n"
            ));
        }
        if file
            .virtual_path
            .extension()
            .is_some_and(|extension| extension == "tsx")
            && shadow_path
                .extension()
                .is_none_or(|extension| extension != "tsx")
        {
            let source_name = file
                .original_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| crate::batch::error::CorsaError::PathError {
                    path: file.original_path.clone(),
                })?;
            return Ok(cstr!(
                "export {{ default }} from \"./{source_name}.tsx\";\nexport * from \"./{source_name}.tsx\";\n"
            ));
        }
        // Corsa is explicitly activated on the Canon mirror tsconfig before
        // these files are opened, so the shared-helper ambient belongs to the
        // same native project. Keep ordinary Vue companions byte-identical to
        // their canonical generated module. The explicit forwarders above and
        // authored TS/declaration shadows deliberately fail closed for
        // canonical coordinates when their bytes differ.
        Ok(file.content.clone())
    }

    pub(super) fn package_shadow_dependency_links(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> Vec<PackageNodeModulesLink> {
        let mut scan = DependencyLinkScan::new(expected_files, &self.package_shadow_manifests);
        for binding in self.package_routes.values() {
            let Some(route) = binding.route.as_ref() else {
                continue;
            };
            scan.walk(route);
        }
        scan.finish()
    }
}

struct DependencyLinkScan<'a> {
    files: MaterializedPrefixIndex<'a>,
    shadows_by_manifest: FxHashMap<&'a Path, Vec<&'a Path>>,
    seen_routes: FxHashSet<(PathBuf, PathBuf)>,
    seen_links: FxHashSet<(PathBuf, PathBuf)>,
    node_modules_dirs: FxHashMap<PathBuf, bool>,
    links: Vec<PackageNodeModulesLink>,
}

impl<'a> DependencyLinkScan<'a> {
    fn new(
        expected_files: &'a FxHashSet<PathBuf>,
        manifests: &'a FxHashMap<PathBuf, PathBuf>,
    ) -> Self {
        let mut shadows_by_manifest: FxHashMap<&Path, Vec<&Path>> = FxHashMap::default();
        for (manifest, original) in manifests {
            let Some(shadow_root) = manifest.parent() else {
                continue;
            };
            shadows_by_manifest
                .entry(original.as_path())
                .or_default()
                .push(shadow_root);
        }
        Self {
            files: MaterializedPrefixIndex::from_files(expected_files.iter().map(PathBuf::as_path)),
            shadows_by_manifest,
            seen_routes: FxHashSet::default(),
            seen_links: FxHashSet::default(),
            node_modules_dirs: FxHashMap::default(),
            links: Vec::new(),
        }
    }

    fn walk(&mut self, route: &crate::PackageRoute) {
        if !self
            .seen_routes
            .insert((route.package_root.clone(), route.manifest_path.clone()))
        {
            return;
        }
        self.push_route_links(route);
        for nested in &route.nested_routes {
            self.walk(nested);
        }
    }

    fn push_route_links(&mut self, route: &crate::PackageRoute) {
        if !self.node_modules_dir(route) {
            return;
        }
        let Some(shadow_roots) = self
            .shadows_by_manifest
            .get(route.manifest_path.as_path())
            .cloned()
        else {
            return;
        };
        let real_dir = route.package_root.join("node_modules");
        for shadow_root in shadow_roots {
            let virtual_dir = shadow_root.join("node_modules");
            if !self
                .seen_links
                .insert((virtual_dir.clone(), real_dir.clone()))
            {
                continue;
            }
            let members = self.files.descendants(&virtual_dir).to_vec();
            self.links.extend(merge_aware_node_modules_links(
                &real_dir,
                &virtual_dir,
                members.iter().copied(),
            ));
        }
    }

    fn node_modules_dir(&mut self, route: &crate::PackageRoute) -> bool {
        if let Some(exists) = self.node_modules_dirs.get(&route.package_root) {
            return *exists;
        }
        let exists = route.package_root.join("node_modules").is_dir();
        self.node_modules_dirs
            .insert(route.package_root.clone(), exists);
        exists
    }

    fn finish(mut self) -> Vec<PackageNodeModulesLink> {
        self.links.sort_by(|left, right| {
            (&left.virtual_dir, &left.real_dir).cmp(&(&right.virtual_dir, &right.real_dir))
        });
        self.links.dedup();
        self.links
    }
}

fn declaration_shadow_forwarder(shadow_path: &Path) -> Option<CompactString> {
    let name = shadow_path.file_name()?.to_str()?;
    let target = if let Some(stem) = name.strip_suffix(".d.mts") {
        cstr!("{stem}.mjs")
    } else if let Some(stem) = name.strip_suffix(".d.cts") {
        cstr!("{stem}.cjs")
    } else {
        let stem = name.strip_suffix(".d.ts")?;
        cstr!("{stem}.js")
    };
    Some(cstr!("export * from \"./{target}\";\n"))
}

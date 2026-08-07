//! Materializing the virtual project to disk so Corsa can observe it as a real
//! TypeScript project: writing virtual files, passthrough modules, ambient stub
//! `.d.ts` files, and pruning stale entries from a previous run.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vize_carton::{FxHashSet, profile};

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::{
    ensure_dir, ensure_materialize_root, prune_unexpected_entries, write_if_changed,
};
use crate::batch::runtime_deps::materialize_runtime_dependencies;

use super::package_node_modules::{
    PackageNodeModulesLink, materialize_package_node_modules, package_node_modules_links,
};
use super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUBS_FILE, PACKAGE_BOUNDARY_FILE,
    SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject,
};

impl VirtualProject {
    /// Materialize the virtual project to disk for diagnostics collection.
    ///
    /// The materialized tree is a cache, but Corsa observes it as a real project.
    /// We therefore prune only entries outside the expected file/dir set and
    /// preserve nested runtime dependencies under `node_modules`. Directory
    /// creation is de-duplicated per parent path, and every file write goes
    /// through `write_if_changed`: warm reruns with unchanged content skip the
    /// rewrite entirely, which avoids needless write IO and keeps mtimes stable
    /// so TypeScript's own filesystem caches are not invalidated.
    pub fn materialize(&self) -> CorsaResult<()> {
        let expected_files = self.expected_materialized_files();
        let package_links = self.package_node_modules_links(&expected_files);
        profile!(
            "canon.project.prepare_dir",
            ensure_materialize_root(&self.virtual_root)
        )?;

        profile!(
            "canon.project.gc",
            prune_unexpected_entries(
                &self.virtual_root,
                &expected_files,
                &self.preserved_prune_roots(&package_links)
            )
        )?;

        profile!(
            "canon.project.runtime_deps",
            materialize_runtime_dependencies(&self.project_root, &self.virtual_root)
        )?;

        profile!(
            "canon.project.package_deps",
            materialize_package_node_modules(&package_links)
        );

        profile!(
            "canon.project.write_package_boundary",
            self.write_package_boundary()
        )?;

        profile!(
            "canon.project.write_files",
            (|| -> CorsaResult<()> {
                // Directory creation stays sequential and de-duplicated; the file
                // writes themselves fan out across rayon workers because each
                // write is an independent syscall-bound operation and large
                // projects pay hundreds of milliseconds when they run serially.
                let mut created_dirs: FxHashSet<&Path> = FxHashSet::default();
                for file in self.virtual_files.values() {
                    if let Some(parent) = file.virtual_path.parent()
                        && created_dirs.insert(parent)
                    {
                        ensure_dir(parent)?;
                    }
                }
                for virtual_path in self.passthrough_files.keys() {
                    if let Some(parent) = virtual_path.parent()
                        && created_dirs.insert(parent)
                    {
                        ensure_dir(parent)?;
                    }
                }

                // `write_if_changed` records IO counters per call: actually
                // performed writes land in `io.write.*` (the curator audit
                // consumes actually-written bytes), skipped same-content
                // rewrites in `io.write.skipped.*`.
                self.virtual_files
                    .par_iter()
                    .try_for_each(|(_, file)| -> CorsaResult<()> {
                        write_if_changed(&file.virtual_path, file.content.as_bytes())?;
                        Ok(())
                    })?;
                self.passthrough_files.par_iter().try_for_each(
                    |(virtual_path, original_path)| -> CorsaResult<()> {
                        let content = std::fs::read(original_path)?;
                        write_if_changed(virtual_path, &content)?;
                        Ok(())
                    },
                )?;
                Ok(())
            })()
        )?;

        profile!(
            "canon.project.write_auto_imports",
            self.write_auto_import_stubs()
        )?;

        profile!(
            "canon.project.write_module_augmentations",
            self.write_module_augmentation_stubs()
        )?;

        profile!(
            "canon.project.write_vue_module_stubs",
            self.write_vue_module_stubs()
        )?;

        if self.uses_shared_helpers() {
            profile!(
                "canon.project.write_shared_helpers",
                self.write_shared_helpers()
            )?;
        }

        profile!(
            "canon.project.write_tsconfig",
            self.write_tsconfig_file(&self.virtual_root.join("tsconfig.json"), None, false)
        )?;
        Ok(())
    }

    fn write_package_boundary(&self) -> CorsaResult<()> {
        let content = b"{\n  \"type\": \"module\"\n}\n";
        write_if_changed(&self.virtual_root.join(PACKAGE_BOUNDARY_FILE), content)?;
        Ok(())
    }

    /// Write a declaration-emitting tsconfig and return its path.
    pub fn write_declaration_tsconfig(
        &self,
        out_dir: &Path,
        declaration_map: bool,
    ) -> CorsaResult<PathBuf> {
        let config_path = self.virtual_root.join("tsconfig.declaration.json");
        self.rewrite_tsx_vue_declaration_inputs()?;
        let include_paths = self.declaration_emit_include_paths();
        profile!(
            "canon.project.write_dts_tsconfig",
            self.write_tsconfig_file_with_includes(
                &config_path,
                Some(out_dir),
                declaration_map,
                Some(&include_paths),
            )
        )?;
        Ok(config_path)
    }

    /// The real `node_modules` directories mirrored into the virtual project so
    /// bare specifiers keep resolving from a nested package (#3366).
    fn package_node_modules_links(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> Vec<PackageNodeModulesLink> {
        package_node_modules_links(
            &self.project_root,
            &self.virtual_root,
            expected_files.iter().map(PathBuf::as_path),
        )
    }

    /// Directories the garbage collector must leave alone: the runtime
    /// dependency mirror plus every mirrored nested `node_modules`. Descending
    /// into a mirrored `node_modules` would delete real dependencies, because
    /// those paths resolve to the user's own install.
    fn preserved_prune_roots(&self, package_links: &[PackageNodeModulesLink]) -> Vec<PathBuf> {
        let mut roots = Vec::with_capacity(package_links.len() + 1);
        roots.push(self.virtual_root.join("node_modules"));
        roots.extend(package_links.iter().map(|link| link.virtual_dir.clone()));
        roots
    }

    fn expected_materialized_files(&self) -> FxHashSet<PathBuf> {
        let mut files = FxHashSet::default();
        files.reserve(self.virtual_files.len() + 4);
        files.extend(self.virtual_files.keys().cloned());
        files.extend(self.passthrough_files.keys().cloned());
        if self.has_global_auto_import_stubs() {
            files.insert(self.virtual_root.join(AUTO_IMPORT_STUBS_FILE));
        }
        if self.has_module_augmentation_stubs() {
            files.insert(self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE));
        }
        files.insert(self.virtual_root.join(VUE_MODULE_STUBS_FILE));
        if self.uses_shared_helpers() {
            files.insert(self.virtual_root.join(SHARED_HELPERS_FILE));
        }
        files.insert(self.virtual_root.join(PACKAGE_BOUNDARY_FILE));
        files.insert(self.virtual_root.join("tsconfig.json"));
        files
    }

    pub(super) fn common_virtual_source_dir(&self) -> PathBuf {
        let mut parents = self
            .virtual_files
            .keys()
            .filter_map(|path| path.parent().map(Path::to_path_buf));
        let Some(mut common) = parents.next() else {
            return self.virtual_root.clone();
        };

        for parent in parents {
            while !parent.starts_with(&common) {
                if !common.pop() {
                    return self.virtual_root.clone();
                }
            }
        }

        common
    }

    pub(super) fn resolved_tsconfig_path(&self) -> Option<PathBuf> {
        if let Some(ref tsconfig_path) = self.tsconfig_path {
            return Some(tsconfig_path.clone());
        }

        let tsconfig = self.project_root.join("tsconfig.json");
        tsconfig.exists().then_some(tsconfig)
    }

    /// The configs whose contents govern this project's alias map: the
    /// anchored tsconfig plus, for a solution-style shell, everything it
    /// references (#3923 cache invalidation).
    pub(crate) fn governing_config_paths(&self) -> Vec<PathBuf> {
        let Some(anchored) = self.resolved_tsconfig_path() else {
            return Vec::new();
        };
        let mut paths = vec![anchored.clone()];
        paths.extend(super::tsconfig_gen::references::referenced_project_configs(
            &anchored,
        ));
        paths
    }

    /// File that project-level (file-less) diagnostics are attributed to:
    /// the effective tsconfig when one exists, otherwise the project root.
    pub(crate) fn project_diagnostics_anchor(&self) -> PathBuf {
        self.resolved_tsconfig_path()
            .unwrap_or_else(|| self.project_root.clone())
    }
}

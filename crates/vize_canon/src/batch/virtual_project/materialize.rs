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

use super::package_node_modules::materialize_package_node_modules;
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
        self.materialize_snapshot(&FxHashSet::default(), &Default::default(), None)
            .map(|_| ())
    }

    pub(crate) fn materialize_editor_union(
        &self,
        preserved_files: &FxHashSet<PathBuf>,
        preserved_package_links: &vize_carton::FxHashMap<PathBuf, PathBuf>,
        query_paths: &[PathBuf],
    ) -> CorsaResult<super::MaterializedFileSnapshot> {
        let _lock = super::super::materialize_lock::MaterializeLock::acquire(&self.virtual_root)?;
        let package_links =
            self.materialize_snapshot(preserved_files, preserved_package_links, Some(query_paths))?;
        let mut expected_files = self.expected_materialized_files();
        expected_files.extend(preserved_files.iter().cloned());
        super::MaterializedFileSnapshot::capture_with_links(&expected_files, &package_links)
    }

    fn materialize_snapshot(
        &self,
        preserved_files: &FxHashSet<PathBuf>,
        preserved_package_links: &vize_carton::FxHashMap<PathBuf, PathBuf>,
        query_paths: Option<&[PathBuf]>,
    ) -> CorsaResult<vize_carton::FxHashMap<PathBuf, PathBuf>> {
        let mut expected_files = self.expected_materialized_files();
        expected_files.extend(preserved_files.iter().cloned());
        let mut desired_package_links = self.desired_package_links_for_files(&expected_files);
        for (path, target) in preserved_package_links {
            desired_package_links
                .entry(path.clone())
                .and_modify(|current| {
                    if target < current {
                        current.clone_from(target);
                    }
                })
                .or_insert_with(|| target.clone());
        }
        let package_links = desired_package_links
            .iter()
            .map(
                |(virtual_dir, real_dir)| super::package_node_modules::PackageNodeModulesLink {
                    virtual_dir: virtual_dir.clone(),
                    real_dir: real_dir.clone(),
                },
            )
            .collect::<Vec<_>>();
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
            materialize_runtime_dependencies(
                &self.project_root,
                &self.virtual_root,
                &self.root_package_shadow_scope_entries(),
            )
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
                for virtual_path in self
                    .package_shadow_files
                    .keys()
                    .chain(self.package_shadow_manifests.keys())
                {
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
                self.package_shadow_files.par_iter().try_for_each(
                    |(shadow_path, canonical_path)| -> CorsaResult<()> {
                        let content = self.package_shadow_content(shadow_path, canonical_path)?;
                        write_if_changed(shadow_path, content.as_bytes())?;
                        Ok(())
                    },
                )?;
                self.package_shadow_manifests.par_iter().try_for_each(
                    |(shadow_path, original_path)| -> CorsaResult<()> {
                        let content = std::fs::read(original_path)?;
                        write_if_changed(shadow_path, &content)?;
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

        profile!("canon.project.write_tsconfig", {
            let path = self.virtual_root.join("tsconfig.json");
            if let Some(query_paths) = query_paths {
                let includes = query_paths.iter().map(PathBuf::as_path).collect::<Vec<_>>();
                self.write_tsconfig_file_with_includes(
                    &path,
                    None,
                    false,
                    Some(includes.as_slice()),
                )
            } else {
                self.write_tsconfig_file(&path, None, false)
            }
        })?;
        Ok(desired_package_links)
    }

    pub(super) fn write_package_boundary(&self) -> CorsaResult<()> {
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

    pub(crate) fn expected_materialized_files(&self) -> FxHashSet<PathBuf> {
        let mut files = FxHashSet::default();
        files.reserve(self.virtual_files.len() + 4);
        files.extend(self.virtual_files.keys().cloned());
        files.extend(self.passthrough_files.keys().cloned());
        files.extend(self.package_shadow_files.keys().cloned());
        files.extend(self.package_shadow_manifests.keys().cloned());
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
        let mut config_roots = vec![anchored.clone()];
        config_roots.extend(super::tsconfig_gen::references::referenced_project_configs(
            &anchored,
        ));
        let mut paths = config_roots.clone();
        for config in config_roots {
            if let Ok(flattened) = self.load_compiler_options_flattened(Some(&config)) {
                paths.extend(flattened.input_paths);
            }
        }
        paths.sort();
        paths.dedup();
        paths
    }

    /// File that project-level (file-less) diagnostics are attributed to:
    /// the effective tsconfig when one exists, otherwise the project root.
    pub(crate) fn project_diagnostics_anchor(&self) -> PathBuf {
        self.resolved_tsconfig_path()
            .unwrap_or_else(|| self.project_root.clone())
    }
}

//! Materializing the virtual project to disk so Corsa can observe it as a real
//! TypeScript project: writing virtual files, passthrough modules, ambient stub
//! `.d.ts` files, and pruning stale entries from a previous run.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vize_carton::{FxHashSet, String as CompactString, profile};

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::{
    ensure_dir, ensure_materialize_root, prune_unexpected_entries, write_if_changed,
};
use crate::batch::runtime_deps::materialize_runtime_dependencies;

use super::{AUTO_IMPORT_STUBS_FILE, SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject};

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
        profile!(
            "canon.project.prepare_dir",
            ensure_materialize_root(&self.virtual_root)
        )?;

        profile!(
            "canon.project.gc",
            prune_unexpected_entries(
                &self.virtual_root,
                &expected_files,
                &[self.virtual_root.join("node_modules")]
            )
        )?;

        profile!(
            "canon.project.runtime_deps",
            materialize_runtime_dependencies(&self.project_root, &self.virtual_root)
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
            "canon.project.write_vue_module_stubs",
            self.write_vue_module_stubs()
        )?;

        profile!(
            "canon.project.write_shared_helpers",
            self.write_shared_helpers()
        )?;

        profile!(
            "canon.project.write_tsconfig",
            self.write_tsconfig_file(&self.virtual_root.join("tsconfig.json"), None, false)
        )?;
        Ok(())
    }

    /// Write a declaration-emitting tsconfig and return its path.
    pub fn write_declaration_tsconfig(
        &self,
        out_dir: &Path,
        declaration_map: bool,
    ) -> CorsaResult<PathBuf> {
        let config_path = self.virtual_root.join("tsconfig.declaration.json");
        profile!(
            "canon.project.write_dts_tsconfig",
            self.write_tsconfig_file(&config_path, Some(out_dir), declaration_map)
        )?;
        Ok(config_path)
    }
    fn write_auto_import_stubs(&self) -> CorsaResult<()> {
        if self.virtual_ts_options.auto_import_stubs.is_empty() {
            return Ok(());
        }

        let capacity = self
            .virtual_ts_options
            .auto_import_stubs
            .iter()
            .fold(64usize, |acc, stub| acc + stub.len() + 1);
        let mut content = CompactString::with_capacity(capacity);
        content.push_str("// @ts-nocheck\n");
        content.push_str("// Framework-provided globals for the virtual project.\n");
        for stub in &self.virtual_ts_options.auto_import_stubs {
            content.push_str(stub);
            content.push('\n');
        }

        write_if_changed(
            &self.virtual_root.join(AUTO_IMPORT_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    fn write_vue_module_stubs(&self) -> CorsaResult<()> {
        let content = "// Vue SFC modules resolve through materialized .vue.ts files.\n";
        write_if_changed(
            &self.virtual_root.join(VUE_MODULE_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    /// Write the shared ambient helpers file. The generated `.vue.ts` modules
    /// hoist their common preamble (ImportMeta augmentation, type helpers,
    /// compiler-macro signatures) into this single program-wide declaration.
    fn write_shared_helpers(&self) -> CorsaResult<()> {
        write_if_changed(
            &self.virtual_root.join(SHARED_HELPERS_FILE),
            crate::virtual_ts::SHARED_PREAMBLE_DTS.as_bytes(),
        )?;
        Ok(())
    }

    fn expected_materialized_files(&self) -> FxHashSet<PathBuf> {
        let mut files = FxHashSet::default();
        files.reserve(self.virtual_files.len() + 4);
        files.extend(self.virtual_files.keys().cloned());
        files.extend(self.passthrough_files.keys().cloned());
        if !self.virtual_ts_options.auto_import_stubs.is_empty() {
            files.insert(self.virtual_root.join(AUTO_IMPORT_STUBS_FILE));
        }
        files.insert(self.virtual_root.join(VUE_MODULE_STUBS_FILE));
        files.insert(self.virtual_root.join(SHARED_HELPERS_FILE));
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

    /// File that project-level (file-less) diagnostics are attributed to:
    /// the effective tsconfig when one exists, otherwise the project root.
    pub(crate) fn project_diagnostics_anchor(&self) -> PathBuf {
        self.resolved_tsconfig_path()
            .unwrap_or_else(|| self.project_root.clone())
    }
}

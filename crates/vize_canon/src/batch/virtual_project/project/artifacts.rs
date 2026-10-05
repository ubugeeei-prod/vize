//! Per-authored-source artifact ownership inside a persistent virtual project.

use std::path::{Path, PathBuf};

use super::super::build::RegisteredFile;
use super::super::{SourceArtifacts, VirtualFile, VirtualProject};

const MUSEA_DEFINE_ART_STUB: &str =
    "declare function defineArt(source: string, options?: Record<string, any>): void;";

impl VirtualProject {
    pub(in crate::batch::virtual_project) fn is_package_route_path(&self, path: &Path) -> bool {
        self.package_routes
            .values()
            .filter_map(|binding| binding.route.as_ref())
            .any(|route| {
                route
                    .all_source_paths()
                    .into_iter()
                    .any(|source| source == path)
            })
    }

    pub(super) fn absorb_registered_file(&mut self, registered: RegisteredFile) {
        let shared_helpers_before = self.uses_shared_helpers();
        self.incremental_source_nodes_rebuilt += 1;
        let original_path = registered.file.original_path.clone();
        let previous_materialized = self.remove_registered_source_inner(&original_path);
        if registered.typed_router_import {
            self.typed_router
                .import_files
                .insert(registered.file.virtual_path.clone());
        }
        let auto_import_count = self.virtual_ts_options.auto_import_stubs.len();
        let mut artifacts = SourceArtifacts::default();
        if is_musea_art_vue_path(&registered.file.original_path)
            && !self
                .virtual_ts_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.contains("defineArt"))
        {
            self.virtual_ts_options
                .auto_import_stubs
                .push(MUSEA_DEFINE_ART_STUB.into());
        }
        self.diagnostics.extend(registered.diagnostics);
        self.original_index.insert(
            registered.file.original_path.clone(),
            registered.file.virtual_path.clone(),
        );
        self.original_contents.insert(
            registered.file.virtual_path.clone(),
            registered.original_content,
        );
        if let Some(code) = registered.pre_rewrite_code {
            self.pre_rewrite_code
                .insert(registered.file.virtual_path.clone(), code);
        }
        // Re-registration must refresh the classification, not accumulate it.
        self.unchecked_javascript_files
            .remove(&registered.file.virtual_path);
        if registered.unchecked_javascript {
            self.unchecked_javascript_files
                .insert(registered.file.virtual_path.clone());
        }
        for (virtual_path, original_path) in registered.passthrough_files {
            if !self.virtual_files.contains_key(&virtual_path) {
                artifacts.passthrough_paths.push(virtual_path.clone());
                self.passthrough_files.insert(virtual_path, original_path);
            }
        }
        for file in registered.extra_virtual_files {
            artifacts.virtual_paths.push(file.virtual_path.clone());
            self.passthrough_files.remove(&file.virtual_path);
            self.replace_virtual_file(file);
        }
        artifacts
            .virtual_paths
            .push(registered.file.virtual_path.clone());
        self.passthrough_files.remove(&registered.file.virtual_path);
        // `original_index` maps this source to its own virtual path (inserted above).
        let canonical_virtual = registered.file.virtual_path.clone();
        self.replace_virtual_file(registered.file);
        self.index_package_source(&original_path, &canonical_virtual);
        self.mark_package_shadow_source_changed(&canonical_virtual);
        artifacts.virtual_paths.sort();
        artifacts.virtual_paths.dedup();
        artifacts.passthrough_paths.sort();
        artifacts.passthrough_paths.dedup();
        let mut new_materialized = artifacts
            .virtual_paths
            .iter()
            .chain(artifacts.passthrough_paths.iter())
            .cloned()
            .collect::<Vec<_>>();
        new_materialized.sort();
        for path in &new_materialized {
            self.track_materialized_link_path(path);
        }
        self.source_artifacts.insert(original_path, artifacts);
        self.incremental_materialized_candidates
            .extend(new_materialized.iter().cloned());
        let mut previous_shape = previous_materialized;
        previous_shape.sort();
        if previous_shape != new_materialized {
            self.mark_incremental_config_file();
        }
        self.reconcile_shared_helper_membership(shared_helpers_before);
        if auto_import_count != self.virtual_ts_options.auto_import_stubs.len() {
            self.mark_incremental_stub_files();
        }
    }

    fn replace_virtual_file(&mut self, file: VirtualFile) {
        self.shared_helper_source_count += usize::from(file.source_map.sfc_map.is_some());
        let previous = self.virtual_files.insert(file.virtual_path.clone(), file);
        self.shared_helper_source_count -=
            usize::from(previous.is_some_and(|file| file.source_map.sfc_map.is_some()));
    }

    fn reconcile_shared_helper_membership(&mut self, before: bool) {
        if before != self.uses_shared_helpers() {
            self.incremental_materialized_candidates
                .insert(self.shared_helpers_path());
            self.mark_incremental_config_file();
        }
    }

    pub(crate) fn remove_registered_source(&mut self, original_path: &Path) -> Vec<PathBuf> {
        let before = self.uses_shared_helpers();
        let removed = self.remove_registered_source_inner(original_path);
        self.reconcile_shared_helper_membership(before);
        removed
    }

    fn remove_registered_source_inner(&mut self, original_path: &Path) -> Vec<PathBuf> {
        let canonical = vize_carton::path::canonicalize_non_verbatim(original_path);
        let key = if self.source_artifacts.contains_key(original_path) {
            original_path
        } else {
            canonical.as_path()
        };
        let Some(artifacts) = self.source_artifacts.remove(key) else {
            return Vec::new();
        };
        if let Some(canonical_virtual) = self.original_index.get(key).cloned() {
            self.mark_package_shadow_source_changed(&canonical_virtual);
        }
        self.remove_package_source_from_index(key);
        self.original_index.remove(key);
        self.diagnostics.retain(|diagnostic| diagnostic.file != key);
        let mut removed =
            Vec::with_capacity(artifacts.virtual_paths.len() + artifacts.passthrough_paths.len());
        for path in artifacts.virtual_paths {
            self.shared_helper_source_count -= usize::from(
                self.virtual_files
                    .remove(&path)
                    .is_some_and(|file| file.source_map.sfc_map.is_some()),
            );
            self.typed_router.import_files.remove(&path);
            self.original_contents.remove(&path);
            self.pre_rewrite_code.remove(&path);
            self.unchecked_javascript_files.remove(&path);
            removed.push(path);
        }
        for path in artifacts.passthrough_paths {
            self.passthrough_files.remove(&path);
            removed.push(path);
        }
        self.incremental_materialized_candidates
            .extend(removed.iter().cloned());
        for path in &removed {
            self.untrack_materialized_link_path(path);
        }
        removed
    }
}

fn is_musea_art_vue_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".art.vue"))
}

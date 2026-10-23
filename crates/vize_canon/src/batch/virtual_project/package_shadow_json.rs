//! Relative JSON modules imported by a package source the virtual project
//! already includes.
//!
//! A root input copies those modules next to its mirror (`passthrough`).
//! Dependency sources are checked through an importer shadow that otherwise
//! keeps only registered sources, so the same relative JSON never lands beside
//! the shadowed file and Corsa reports TS2307 (#10350). The bytes are raw
//! copies, owned with the shadow the same way `package.json` is.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::super::VirtualProject;
use super::super::passthrough::collect_relative_json_modules;
use super::PackageShadowTopology;

impl VirtualProject {
    pub(super) fn collect_package_json_module_shadows(
        &self,
        source_root: &Path,
        shadow_root: &Path,
        topology: &mut PackageShadowTopology,
    ) {
        let normalized_root = vize_carton::path::canonicalize_non_verbatim(source_root);
        let Some(indexed) = self.package_source_index.get(&normalized_root) else {
            return;
        };
        // Most package sources import no JSON; skip the shadow scan for them.
        let mut json_sources = indexed
            .iter()
            .filter_map(|(source, (_, virtual_path))| {
                let content = self.original_contents.get(virtual_path)?;
                let json_files = collect_relative_json_modules(source, content);
                (!json_files.is_empty()).then_some((source, virtual_path, json_files))
            })
            .collect::<Vec<_>>();
        if json_sources.is_empty() {
            return;
        }
        json_sources.sort_by(|left, right| left.0.cmp(right.0));
        let mut shadow_files = topology
            .files
            .iter()
            .filter(|(destination, _)| destination.starts_with(shadow_root))
            .map(|(destination, canonical)| (destination.clone(), canonical.clone()))
            .collect::<Vec<_>>();
        shadow_files.sort();

        for (source, virtual_path, json_files) in json_sources {
            let Some(authored_name) = source.file_name() else {
                continue;
            };
            for (destination, canonical) in &shadow_files {
                if canonical != virtual_path {
                    continue;
                }
                let Some(shadow_name) = destination.file_name() else {
                    continue;
                };
                if !shadow_carries_authored_imports(authored_name, shadow_name) {
                    continue;
                }
                for json_file in &json_files {
                    let Some(json_destination) =
                        json_shadow_destination(destination, source, json_file, shadow_root)
                    else {
                        continue;
                    };
                    if topology.files.contains_key(&json_destination) {
                        continue;
                    }
                    topology
                        .manifests
                        .entry(json_destination)
                        .or_insert_with(|| json_file.clone());
                }
            }
        }
    }
}

/// `.vue` sources are shadowed as `.vue.ts` / `.vue.tsx` companions. Those
/// companions keep the authored relative import, so the JSON sits beside them.
fn shadow_carries_authored_imports(authored_name: &OsStr, shadow_name: &OsStr) -> bool {
    if authored_name == shadow_name {
        return true;
    }
    let (Some(authored), Some(shadow)) = (authored_name.to_str(), shadow_name.to_str()) else {
        return false;
    };
    if !authored.ends_with(".vue") {
        return false;
    }
    shadow == cstr!("{authored}.ts").as_str() || shadow == cstr!("{authored}.tsx").as_str()
}

fn json_shadow_destination(
    shadow_file: &Path,
    authored_file: &Path,
    json_file: &Path,
    shadow_root: &Path,
) -> Option<PathBuf> {
    let relative = relative_path_from(authored_file.parent()?, json_file)?;
    let destination = normalize_path_lexically(&shadow_file.parent()?.join(relative));
    (destination.starts_with(shadow_root) && destination != shadow_root).then_some(destination)
}

fn relative_path_from(from_dir: &Path, target: &Path) -> Option<PathBuf> {
    let from = path_components(from_dir);
    let to = path_components(target);
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return None;
    }
    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push("..");
    }
    for component in to.iter().skip(common) {
        relative.push(component);
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

fn path_components(path: &Path) -> Vec<std::ffi::OsString> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::CurDir => None,
            component => Some(component.as_os_str().to_os_string()),
        })
        .collect()
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

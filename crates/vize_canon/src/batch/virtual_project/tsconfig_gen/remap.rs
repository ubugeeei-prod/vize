//! Re-anchoring of tsconfig path-like options into the virtual mirror.

use std::path::Path;

use serde_json::{Map, Value};
use vize_carton::{String as CompactString, cstr};

use super::super::VirtualProject;
use super::control_alias::protect_control_file_aliases;
use super::vue_alias::remap_path_targets;

impl VirtualProject {
    /// Re-anchor tsconfig `paths` targets into the virtual mirror. Each relative
    /// target yields two candidates: the mirror copy (resolved relative to the
    /// virtual tsconfig, which lives in the mirror root) followed by the real
    /// source-tree path as a fallback, plus a trailing `.vue.ts` mirror
    /// candidate so extensionless SFC aliases resolve (see [`remap_path_targets`]).
    /// Absolute and non-string targets pass through unchanged.
    #[allow(clippy::disallowed_types)]
    pub(super) fn remap_paths(
        &self,
        paths: &Map<std::string::String, Value>,
    ) -> Map<std::string::String, Value> {
        let up = self.virtual_root_to_project_prefix();
        let mut remapped = Map::new();
        for (alias, targets) in paths {
            let Some(targets) = targets.as_array() else {
                remapped.insert(alias.clone(), targets.clone());
                continue;
            };
            remapped.insert(
                alias.clone(),
                Value::Array(remap_path_targets(targets, &up)),
            );
        }
        protect_control_file_aliases(paths, &mut remapped, &up);
        remapped
    }

    /// Add exact bare-specifier routes for workspace package sources whose
    /// manifests point at `.vue` (or at a barrel that reaches one). The target
    /// is already inside the virtual tree, so it must be added after ordinary
    /// project-relative `paths` rebasing. Authored source keeps the bare
    /// package spelling, which is also the spelling declaration emit retains.
    #[allow(clippy::disallowed_types)]
    pub(super) fn insert_virtual_module_alias_paths(
        &self,
        paths: &mut Map<std::string::String, Value>,
    ) {
        let mut aliases: Vec<_> = self.virtual_module_aliases.iter().collect();
        aliases.sort_by_key(|(specifier, _)| specifier.as_str());
        for (specifier, source_paths) in aliases {
            let mut targets = source_paths
                .iter()
                .filter_map(|source_path| self.find_by_original(source_path))
                .filter_map(|file| file.virtual_path.strip_prefix(&self.virtual_root).ok())
                .map(|relative| Value::String(cstr!("./{}", relative.display()).into()))
                .collect::<Vec<_>>();
            if let Some(existing) = paths.get(specifier.as_str()).and_then(Value::as_array) {
                for target in existing {
                    if !targets.contains(target) {
                        targets.push(target.clone());
                    }
                }
            }
            if !targets.is_empty() {
                paths.insert(specifier.as_str().into(), Value::Array(targets));
            }
        }
    }

    /// Re-anchor a list of project-root-relative directories (e.g. `typeRoots`)
    /// into the virtual mirror: each relative entry yields the mirror copy
    /// followed by the real source-tree directory. Absolute and non-string
    /// entries pass through unchanged.
    pub(super) fn remap_dir_entries(&self, entries: &[Value]) -> Vec<Value> {
        let up = self.virtual_root_to_project_prefix();
        let mut remapped = Vec::with_capacity(entries.len() * 2);
        for entry in entries {
            let Some(entry_str) = entry.as_str() else {
                remapped.push(entry.clone());
                continue;
            };
            if Path::new(entry_str).is_absolute() {
                remapped.push(Value::String(entry_str.to_owned()));
                continue;
            }
            let core = entry_str.strip_prefix("./").unwrap_or(entry_str);
            remapped.push(Value::String(cstr!("./{core}").into()));
            remapped.push(Value::String(cstr!("{up}{core}").into()));
        }
        remapped
    }

    /// Relative prefix (e.g. `../../../`) from the virtual root back to the
    /// project root, used to aim alias fallbacks at the real source tree.
    fn virtual_root_to_project_prefix(&self) -> CompactString {
        let depth = self
            .virtual_root
            .strip_prefix(&self.project_root)
            .map(|relative| relative.components().count())
            .unwrap_or(0);
        let mut prefix = CompactString::with_capacity(depth * 3);
        for _ in 0..depth {
            prefix.push_str("../");
        }
        prefix
    }
}

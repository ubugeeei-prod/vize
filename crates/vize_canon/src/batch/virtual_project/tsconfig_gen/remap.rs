//! Re-anchoring of tsconfig path-like options into the virtual mirror.

use std::path::{Component, Path, PathBuf};

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

    /// Relative prefix from the virtual root back to the project root, used to
    /// aim alias fallbacks at the real source tree. Project-keyed mirrors may
    /// live in a shared physical dependency store, so this must compare both
    /// paths instead of assuming that the mirror is nested under the source.
    fn virtual_root_to_project_prefix(&self) -> CompactString {
        let relative = relative_path_from(&self.virtual_root, &self.project_root);
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if normalized == "." {
            return CompactString::new("");
        }
        let mut prefix = CompactString::from(normalized);
        if !prefix.ends_with('/') {
            prefix.push('/');
        }
        prefix
    }
}

fn relative_path_from(from_dir: &Path, target: &Path) -> PathBuf {
    let from = path_components(from_dir);
    let to = path_components(target);
    let mut common = 0usize;
    while common < from.len() && common < to.len() && from[common] == to[common] {
        common += 1;
    }
    if common == 0 {
        return target.to_path_buf();
    }

    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push("..");
    }
    for component in to.iter().skip(common) {
        relative.push(component);
    }
    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn path_components(path: &Path) -> Vec<std::ffi::OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_os_string()),
            Component::RootDir => Some(std::path::MAIN_SEPARATOR_STR.into()),
            Component::ParentDir => Some("..".into()),
            Component::Normal(value) => Some(value.to_os_string()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::relative_path_from;
    use std::path::{Path, PathBuf};

    #[test]
    fn sibling_storage_paths_rebase_to_the_source_tree() {
        assert_eq!(
            relative_path_from(
                Path::new("/workspace/shared/.vize/canon/projects/key"),
                Path::new("/workspace/apps/first"),
            ),
            PathBuf::from("../../../../../apps/first")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_rebasing_respects_drive_prefix_boundaries() {
        assert_eq!(
            relative_path_from(
                Path::new(r"C:\store\.vize\canon\projects\key"),
                Path::new(r"C:\repo\app"),
            ),
            PathBuf::from(r"..\..\..\..\..\repo\app")
        );
        assert_eq!(
            relative_path_from(
                Path::new(r"C:\store\.vize\canon\projects\key"),
                Path::new(r"D:\repo\app"),
            ),
            PathBuf::from(r"D:\repo\app"),
            "a different drive must remain an absolute paths target"
        );
    }
}

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
            let targets = targets
                .iter()
                .map(|target| self.path_target_onto_project_root(target))
                .collect::<Vec<_>>();
            remapped.insert(
                alias.clone(),
                Value::Array(remap_path_targets(&targets, &up)),
            );
        }
        protect_control_file_aliases(paths, &mut remapped, &up);
        remapped
    }

    /// A self-contained config may spell an in-project target absolutely so it
    /// can live outside the source tree. Translate that target back onto the
    /// project root before mirror expansion; only genuinely escaped paths
    /// belong in `__vize_external__`.
    fn path_target_onto_project_root(&self, target: &Value) -> Value {
        let Some(raw) = target.as_str() else {
            return target.clone();
        };
        let path = Path::new(raw);
        if !path.is_absolute() {
            return target.clone();
        }
        let normalized = super::super::tsconfig_paths::normalize_path_lexically(path);
        let Ok(relative) = normalized.strip_prefix(&self.project_root) else {
            return target.clone();
        };
        Value::String(relative.to_string_lossy().replace('\\', "/"))
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
    use super::{VirtualProject, relative_path_from};
    use serde_json::{Value, json};
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

    #[test]
    fn absolute_in_project_targets_return_to_the_local_mirror() {
        let case = tempfile::tempdir().unwrap();
        let project = VirtualProject::new(case.path()).unwrap();
        let target = project.project_root().join("src/*");

        assert_eq!(
            project.path_target_onto_project_root(&Value::String(
                target.to_string_lossy().into_owned()
            )),
            json!("src/*")
        );
        assert_eq!(
            project.path_target_onto_project_root(&json!("/outside/src/*")),
            json!("/outside/src/*")
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

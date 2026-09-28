//! Hidden source directories explicitly named by TypeScript configuration.

use std::path::PathBuf;

use vize_carton::FxHashSet;

use crate::batch::virtual_project::{VirtualProject, parse_jsonc_value};

/// TypeScript includes explicitly listed hidden files (notably Nuxt's
/// `.nuxt`), but walking every hidden cache or VCS directory is wasteful.
pub(super) fn explicit_hidden_source_dirs(project: &VirtualProject) -> FxHashSet<PathBuf> {
    let root = project.project_root();
    let mut directories = FxHashSet::default();
    for config_path in project.governing_config_paths() {
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(config) = parse_jsonc_value(&content) else {
            continue;
        };
        for entry in ["files", "include"]
            .into_iter()
            .filter_map(|key| config.get(key).and_then(serde_json::Value::as_array))
            .flatten()
            .filter_map(serde_json::Value::as_str)
        {
            let Some(base) = config_path.parent() else {
                continue;
            };
            let path = vize_carton::path::canonicalize_non_verbatim(&base.join(entry));
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let mut ancestor = root.to_path_buf();
            for component in relative.components() {
                ancestor.push(component);
                if component.as_os_str().to_string_lossy().starts_with('.') {
                    directories.insert(ancestor.clone());
                }
            }
        }
    }
    directories
}

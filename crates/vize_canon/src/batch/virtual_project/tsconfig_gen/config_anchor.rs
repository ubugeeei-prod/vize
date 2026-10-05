//! Preserve the governing config's type-reference lookup directory (#7825).

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::String;

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::{ensure_dir, write_if_changed};

use super::VirtualProject;

impl VirtualProject {
    /// The config stays beside its mirrored source directory even when imports
    /// widen the project root. TypeScript resolves `types` and implicit @types
    /// from this directory, independently of the program's common source root.
    pub(crate) fn generated_tsconfig_path(&self) -> PathBuf {
        let directory = self
            .resolved_tsconfig_path()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(&path))
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .and_then(|directory| {
                directory
                    .strip_prefix(&self.project_root)
                    .ok()
                    .map(Path::to_path_buf)
            })
            .unwrap_or_default();
        self.virtual_root.join(directory).join("tsconfig.json")
    }

    pub(super) fn write_generated_config(
        &self,
        path: &Path,
        config: &mut Value,
    ) -> CorsaResult<()> {
        let directory = path.parent().unwrap_or(&self.virtual_root);
        let relative = directory
            .strip_prefix(&self.virtual_root)
            .unwrap_or(Path::new(""));
        let mut prefix = String::default();
        for _ in relative.components() {
            prefix.push_str("../");
        }
        if !prefix.is_empty() {
            reanchor_config(config, &prefix);
        }
        ensure_dir(directory)?;
        let content = serde_json::to_string_pretty(config)?;
        write_if_changed(path, content.as_bytes())?;
        Ok(())
    }
}

fn reanchor_config(config: &mut Value, prefix: &str) {
    if let Some(entries) = config.get_mut("include").and_then(Value::as_array_mut) {
        reanchor_entries(entries, prefix);
    }
    let Some(options) = config
        .get_mut("compilerOptions")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    if let Some(paths) = options.get_mut("paths").and_then(Value::as_object_mut) {
        for targets in paths.values_mut().filter_map(Value::as_array_mut) {
            reanchor_entries(targets, prefix);
        }
    }
    if let Some(entries) = options.get_mut("typeRoots").and_then(Value::as_array_mut) {
        reanchor_entries(entries, prefix);
    }
    for name in ["mapRoot", "sourceRoot", "outFile"] {
        if let Some(value) = options.get_mut(name) {
            reanchor_entry(value, prefix);
        }
    }
}

fn reanchor_entries(entries: &mut [Value], prefix: &str) {
    for entry in entries {
        reanchor_entry(entry, prefix);
    }
}

fn reanchor_entry(entry: &mut Value, prefix: &str) {
    let Some(value) = entry.as_str() else {
        return;
    };
    if Path::new(value).is_absolute() || super::path_rebase::is_url(value) {
        return;
    }
    *entry = Value::String(vize_carton::cstr!("{prefix}{value}").into());
}

#[cfg(test)]
mod tests {
    use super::reanchor_config;
    use serde_json::json;

    #[test]
    fn nested_config_preserves_all_mirror_relative_paths_and_type_names() {
        let outside = std::env::temp_dir().join("outside");
        let outside_src = outside.join("src/*").to_string_lossy().into_owned();
        let outside_types = outside.join("types").to_string_lossy().into_owned();
        let mut config = json!({
            "include": ["app/src/a.ts", "shared/data.ts", "__vize_helpers.d.ts"],
            "compilerOptions": {
                "paths": {"@/*": ["./app/src/*", outside_src]},
                "typeRoots": ["./app/types", "../../app/types", outside_types],
                "types": ["foo", "@example/client"],
                "rootDir": "/mirror", "outDir": "/output",
                "sourceRoot": "https://example.test/source", "outFile": "app/result.js"
            }
        });
        reanchor_config(&mut config, "../");
        assert_eq!(
            config,
            json!({
                "include": ["../app/src/a.ts", "../shared/data.ts", "../__vize_helpers.d.ts"],
                "compilerOptions": {
                "paths": {"@/*": [".././app/src/*", outside_src]},
                "typeRoots": [".././app/types", "../../../app/types", outside_types],
                    "types": ["foo", "@example/client"],
                    "rootDir": "/mirror", "outDir": "/output",
                    "sourceRoot": "https://example.test/source", "outFile": "../app/result.js"
                }
            })
        );
    }
}

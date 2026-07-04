use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use vize_carton::{String, path::canonicalize_non_verbatim};

use crate::batch::{CorsaResult, VirtualProject};

pub(super) fn rewrite_declaration_map_outputs(
    out_dir: &Path,
    project: &VirtualProject,
) -> CorsaResult<()> {
    if !out_dir.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || !is_declaration_map_file(path) {
            continue;
        }

        rewrite_declaration_map(path, project)?;
    }

    Ok(())
}

fn rewrite_declaration_map(path: &Path, project: &VirtualProject) -> CorsaResult<()> {
    let content = fs::read_to_string(path)?;
    let mut map: Value = serde_json::from_str(&content)?;
    let source_root: String = map
        .get("sourceRoot")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .into();
    let Some(sources) = map.get("sources").and_then(Value::as_array) else {
        return Ok(());
    };
    let map_dir = path.parent().unwrap_or_else(|| Path::new("."));

    let rewrites = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            let value = source.as_str()?;
            rewrite_map_source(value, source_root.as_str(), map_dir, project)
                .map(|rewrite| (index, rewrite))
        })
        .collect::<Vec<_>>();

    if rewrites.is_empty() {
        return Ok(());
    }

    if let Some(sources) = map.get_mut("sources").and_then(Value::as_array_mut) {
        for (index, rewrite) in &rewrites {
            sources[*index] = Value::String(rewrite.source.as_str().into());
        }
    }

    if let Some(sources_content) = map.get_mut("sourcesContent").and_then(Value::as_array_mut) {
        for (index, rewrite) in &rewrites {
            let Some(source_content) = rewrite.source_content else {
                continue;
            };
            let Some(value) = sources_content.get_mut(*index) else {
                continue;
            };
            *value = Value::String(source_content.into());
        }
    }

    if let Some(object) = map.as_object_mut() {
        object.insert("sourceRoot".into(), Value::String("".into()));
    }
    fs::write(path, serde_json::to_string(&map)?)?;

    Ok(())
}

struct MapSourceRewrite<'a> {
    source: String,
    source_content: Option<&'a str>,
}

fn rewrite_map_source<'a>(
    source: &str,
    source_root: &str,
    map_dir: &Path,
    project: &'a VirtualProject,
) -> Option<MapSourceRewrite<'a>> {
    let source_path = normalize_path_lexically(&resolve_source_path(source, source_root, map_dir));
    let virtual_path = canonicalize_non_verbatim(&source_path);
    let file = project
        .find_by_virtual(&virtual_path)
        .or_else(|| project.find_by_virtual(&source_path))?;
    let map_dir = canonicalize_non_verbatim(map_dir);
    let original_path = canonicalize_non_verbatim(&file.original_path);
    Some(MapSourceRewrite {
        source: path_to_map_source(&relative_path_from(&map_dir, &original_path)),
        source_content: project.original_content_for_virtual(&file.virtual_path),
    })
}

fn resolve_source_path(source: &str, source_root: &str, map_dir: &Path) -> PathBuf {
    let source = Path::new(source);
    if source.is_absolute() {
        return source.to_path_buf();
    }
    let source_root = Path::new(source_root);
    if source_root.as_os_str().is_empty() {
        map_dir.join(source)
    } else if source_root.is_absolute() {
        source_root.join(source)
    } else {
        map_dir.join(source_root).join(source)
    }
}

fn is_declaration_map_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts.map")
                || name.ends_with(".d.mts.map")
                || name.ends_with(".d.cts.map")
        })
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

fn path_components(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::CurDir => None,
            component => Some(component.as_os_str().to_os_string()),
        })
        .collect()
}

fn path_to_map_source(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::TempDir;
    use vize_carton::path::canonicalize_non_verbatim;

    use super::{path_to_map_source, relative_path_from, rewrite_declaration_map_outputs};
    use crate::batch::VirtualProject;

    #[test]
    fn rewrites_virtual_sources_to_original_sources() {
        let temp = TempDir::new().unwrap();
        let root = canonicalize_non_verbatim(temp.path());
        let src = root.join("src");
        let out_dir = root.join("dist/types");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&out_dir).unwrap();

        let vue_path = src.join("App.vue");
        let index_path = src.join("index.ts");
        let vue_source = r#"<script setup lang="ts">
export interface PublicProps {
  label: string
}
defineProps<PublicProps>()
</script>
"#;
        fs::write(&vue_path, vue_source).unwrap();
        fs::write(&index_path, "export { default as App } from './App.vue'\n").unwrap();

        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&vue_path).unwrap();
        project.register_path(&index_path).unwrap();

        let vue_virtual = project.find_by_original(&vue_path).unwrap();
        let index_virtual = project.find_by_original(&index_path).unwrap();
        fs::write(
            out_dir.join("App.vue.d.ts.map"),
            serde_json::to_string(&json!({
                "version": 3,
                "file": "App.vue.d.ts",
                "sourceRoot": "",
                "sources": [
                    path_to_map_source(&relative_path_from(&out_dir, &vue_virtual.virtual_path))
                ],
                "sourcesContent": [
                    vue_virtual.content.as_str()
                ],
                "names": [],
                "mappings": ""
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            out_dir.join("index.d.ts.map"),
            serde_json::to_string(&json!({
                "version": 3,
                "file": "index.d.ts",
                "sourceRoot": "",
                "sources": [
                    path_to_map_source(&relative_path_from(&out_dir, &index_virtual.virtual_path))
                ],
                "names": [],
                "mappings": ""
            }))
            .unwrap(),
        )
        .unwrap();

        rewrite_declaration_map_outputs(&out_dir, &project).unwrap();

        assert_map_sources(&out_dir.join("App.vue.d.ts.map"), &["../../src/App.vue"]);
        assert_map_source_content(&out_dir.join("App.vue.d.ts.map"), 0, vue_source);
        assert_map_sources(&out_dir.join("index.d.ts.map"), &["../../src/index.ts"]);
    }

    fn assert_map_sources(path: &std::path::Path, expected: &[&str]) {
        let map: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let sources = map["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| source.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(sources, expected);
        assert_eq!(map["sourceRoot"], "");
    }

    fn assert_map_source_content(path: &std::path::Path, index: usize, expected: &str) {
        let map: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let content = map["sourcesContent"][index].as_str().unwrap();
        assert_eq!(content, expected);
    }
}

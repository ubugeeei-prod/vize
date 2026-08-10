mod tsx_shims;

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::cstr;

use crate::batch::CorsaResult;

use super::{VirtualFile, VirtualProject};
use tsx_shims::{is_script_path, rewrite_tsx_vue_shim_specifiers};

impl VirtualProject {
    pub(super) fn rewrite_tsx_vue_declaration_inputs(&self) -> CorsaResult<()> {
        for file in self.virtual_files_sorted() {
            let path = file.virtual_path.as_path();
            if !is_script_path(path) {
                continue;
            }
            let content = std::fs::read_to_string(path)?;
            let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
            let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let Some(rewritten) =
                rewrite_tsx_vue_shim_specifiers(&content, source_type, source_dir)
            else {
                continue;
            };
            std::fs::write(path, rewritten.as_str())?;
        }
        Ok(())
    }

    pub(super) fn declaration_emit_include_paths(&self) -> Vec<&Path> {
        self.virtual_files
            .values()
            .filter(|file| !self.is_tsx_vue_import_shim(file))
            .map(|file| file.virtual_path.as_path())
            .collect()
    }

    fn is_tsx_vue_import_shim(&self, file: &VirtualFile) -> bool {
        if file.original_path != file.virtual_path {
            return false;
        }
        let Some(file_name) = file.virtual_path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        if !file_name.ends_with(".vue.ts") {
            return false;
        }
        let tsx_path = file
            .virtual_path
            .with_file_name(cstr!("{file_name}x").as_str());
        self.virtual_files.contains_key(&tsx_path)
    }

    /// Restore the caller's configured declaration layout after a widened
    /// workspace-package emit, then remove outputs for inferred dependencies.
    /// Authored declarations retain bare package specifiers; the external
    /// mirror and its implementation paths never become publishable output.
    pub(crate) fn finalize_declaration_outputs(
        &self,
        out_dir: &Path,
        config_path: &Path,
    ) -> CorsaResult<()> {
        let config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
        let Some(root_dir) = config["compilerOptions"]["rootDir"].as_str() else {
            return Ok(());
        };
        let emit_root_dir = Path::new(root_dir);
        let layout_root_dir = self
            .configured_declaration_root_dir()?
            .unwrap_or_else(|| emit_root_dir.to_path_buf());
        for file in self.virtual_files.values() {
            if self.is_tsx_vue_import_shim(file) {
                continue;
            }
            let Ok(emit_relative) = file.virtual_path.strip_prefix(emit_root_dir) else {
                continue;
            };
            let emitted = declaration_output_path(out_dir, emit_relative);
            let original = vize_carton::path::canonicalize_non_verbatim(&file.original_path);
            if self.is_declaration_root(&original) {
                let Ok(layout_relative) = file.virtual_path.strip_prefix(&layout_root_dir) else {
                    continue;
                };
                let destination = declaration_output_path(out_dir, layout_relative);
                relocate_declaration_output(&emitted, &destination, out_dir)?;
                continue;
            }
            remove_file_if_present(&emitted)?;
            remove_file_if_present(&declaration_map_path(&emitted))?;
            remove_empty_ancestors(emitted.parent(), out_dir)?;
        }
        Ok(())
    }
}

fn declaration_output_path(out_dir: &Path, relative_input: &Path) -> PathBuf {
    let mut output = out_dir.join(relative_input);
    let extension = match relative_input
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("mts") => "d.mts",
        Some("cts") => "d.cts",
        _ => "d.ts",
    };
    output.set_extension(extension);
    output
}

fn declaration_map_path(declaration: &Path) -> PathBuf {
    let mut map = declaration.as_os_str().to_os_string();
    map.push(".map");
    PathBuf::from(map)
}

fn relocate_declaration_output(
    source: &Path,
    destination: &Path,
    boundary: &Path,
) -> CorsaResult<()> {
    if source == destination || !source.exists() {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    remove_file_if_present(destination)?;
    std::fs::rename(source, destination)?;

    let source_map = declaration_map_path(source);
    let destination_map = declaration_map_path(destination);
    if source_map.exists() {
        rebase_map_sources_for_move(&source_map, &destination_map)?;
        remove_file_if_present(&destination_map)?;
        std::fs::rename(&source_map, &destination_map)?;
    }
    remove_empty_ancestors(source.parent(), boundary)?;
    Ok(())
}

fn rebase_map_sources_for_move(source: &Path, destination: &Path) -> CorsaResult<()> {
    let mut map: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(source)?)?;
    let source_root = map
        .get("sourceRoot")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let source_dir = source.parent().unwrap_or_else(|| Path::new("."));
    let destination_dir = destination.parent().unwrap_or_else(|| Path::new("."));
    if let Some(sources) = map
        .get_mut("sources")
        .and_then(serde_json::Value::as_array_mut)
    {
        for source in sources {
            let Some(raw_source) = source.as_str() else {
                continue;
            };
            let raw_source = Path::new(raw_source);
            let resolved = if raw_source.is_absolute() {
                raw_source.to_path_buf()
            } else if Path::new(&source_root).is_absolute() {
                Path::new(&source_root).join(raw_source)
            } else {
                source_dir.join(&source_root).join(raw_source)
            };
            *source = serde_json::Value::String(
                relative_path_from(destination_dir, &normalize_path_lexically(&resolved))
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    if let Some(object) = map.as_object_mut() {
        object.insert("sourceRoot".into(), serde_json::Value::String("".into()));
    }
    std::fs::write(source, serde_json::to_string(&map)?)?;
    Ok(())
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
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
    relative
}

fn path_components(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            component => Some(component.as_os_str().to_os_string()),
        })
        .collect()
}

fn remove_file_if_present(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_empty_ancestors(mut directory: Option<&Path>, boundary: &Path) -> std::io::Result<()> {
    while let Some(path) = directory {
        if path == boundary || !path.starts_with(boundary) {
            break;
        }
        match std::fs::remove_dir(path) {
            Ok(()) => directory = path.parent(),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) =>
            {
                break;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

//! Nuxt generated directory resolution.

use std::{
    fs,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use serde_json::Value;
use vize_s0::{String, ToCompactString};

use super::parsing::nuxt_config_static_string;
use crate::commands::check::tsconfig_inputs::parse_jsonc_value;

const DECLARATION_SUFFIXES: &[&str] = &[".d.ts", ".d.mts", ".d.cts"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NuxtGeneratedDir {
    path: PathBuf,
    display: String,
}

impl NuxtGeneratedDir {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn display(&self) -> &str {
        self.display.as_str()
    }

    pub(super) fn imports_path(&self) -> PathBuf {
        for suffix in DECLARATION_SUFFIXES {
            let path = self.path.join(format!("imports{suffix}"));
            if path.is_file() {
                return path;
            }
        }
        self.path.join("imports.d.ts")
    }

    pub(super) fn tsconfig_path(&self) -> PathBuf {
        self.path.join("tsconfig.json")
    }

    pub(super) fn types_dir(&self) -> PathBuf {
        self.path.join("types")
    }

    pub(super) fn root_dts_files(&self) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(self.path()) else {
            return Vec::new();
        };

        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && is_declaration_file(path))
            .collect()
    }

    pub(super) fn dts_files(&self) -> Vec<PathBuf> {
        let mut files = self.root_dts_files();
        let types_dir = self.types_dir();
        if types_dir.exists() {
            let walker = WalkBuilder::new(types_dir.as_path())
                .hidden(false)
                .standard_filters(false)
                .build();

            for entry in walker.flatten() {
                let path = entry.path();
                if path.is_file() && is_declaration_file(path) {
                    files.push(path.to_path_buf());
                }
            }
        }
        files
    }
}

pub(super) fn is_declaration_file(path: &Path) -> bool {
    declaration_stem(path).is_some()
}

pub(super) fn is_imports_declaration(path: &Path) -> bool {
    declaration_stem(path) == Some("imports")
}

pub(super) fn is_nitro_imports_declaration(path: &Path) -> bool {
    declaration_stem(path) == Some("nitro-imports")
}

fn declaration_stem(path: &Path) -> Option<&str> {
    let file_name = path.file_name().and_then(|name| name.to_str())?;
    DECLARATION_SUFFIXES
        .iter()
        .find_map(|suffix| file_name.strip_suffix(suffix))
}

pub(super) fn resolve_nuxt_generated_dir(cwd: &Path) -> NuxtGeneratedDir {
    let path = generated_dir_from_nuxt_config(cwd)
        .or_else(|| generated_dir_from_tsconfig_imports(cwd))
        .unwrap_or_else(|| cwd.join(".nuxt"));
    let path = normalize_path_lexically(&path);
    let display = display_path(cwd, &path);
    NuxtGeneratedDir { path, display }
}

fn generated_dir_from_tsconfig_imports(cwd: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(cwd.join("tsconfig.json")).ok()?;
    let value = parse_jsonc_value(content.as_str()).ok()?;
    let paths = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("paths"))
        .and_then(Value::as_object)?;

    for key in ["#imports", "#imports/*"] {
        let Some(targets) = paths.get(key).and_then(Value::as_array) else {
            continue;
        };
        for target in targets {
            let Some(target) = target.as_str() else {
                continue;
            };
            if let Some(dir) = generated_dir_from_imports_target(cwd, target) {
                return Some(dir);
            }
        }
    }

    None
}

fn generated_dir_from_imports_target(cwd: &Path, target: &str) -> Option<PathBuf> {
    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    let mut path = PathBuf::from(target);
    if !path.is_absolute() {
        path = cwd.join(path);
    }

    let file_name = path.file_name().and_then(|name| name.to_str())?;
    let mut dir = match file_name {
        "imports" => path.parent()?.to_path_buf(),
        "*" => {
            let parent = path.parent()?;
            if parent.file_name().and_then(|name| name.to_str()) == Some("imports") {
                parent.parent()?.to_path_buf()
            } else {
                parent.to_path_buf()
            }
        }
        _ if is_imports_declaration(&path) => path.parent()?.to_path_buf(),
        _ => return None,
    };

    if dir.file_name().and_then(|name| name.to_str()) == Some("types")
        && let Some(parent) = dir.parent()
    {
        dir = parent.to_path_buf();
    }

    Some(dir)
}

fn generated_dir_from_nuxt_config(cwd: &Path) -> Option<PathBuf> {
    let build_dir = nuxt_config_static_string(cwd, "buildDir")?;
    let path = PathBuf::from(build_dir.as_str());
    Some(if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    })
}

pub(super) fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn display_path(cwd: &Path, path: &Path) -> String {
    let cwd = normalize_path_lexically(cwd);
    let relative = path.strip_prefix(&cwd).unwrap_or(path);
    let rendered = if relative.as_os_str().is_empty() {
        "."
    } else {
        relative.to_str().unwrap_or_default()
    };
    rendered.replace('\\', "/").to_compact_string()
}

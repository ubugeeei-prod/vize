//! Nuxt generated directory resolution.

use std::{
    fs,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use serde_json::Value;
use vize_carton::{String, ToCompactString};

use super::parsing::{
    default_export_config_object, extract_expression, find_object_property, nuxt_config_source,
};
use crate::commands::check::tsconfig_inputs::parse_jsonc_value;

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
            .filter(|path| {
                path.is_file()
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".d.ts"))
            })
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
                let is_dts = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".d.ts"));
                if path.is_file() && is_dts {
                    files.push(path.to_path_buf());
                }
            }
        }
        files
    }
}

pub(super) fn resolve_nuxt_generated_dir(cwd: &Path) -> NuxtGeneratedDir {
    let path = generated_dir_from_tsconfig_imports(cwd)
        .or_else(|| generated_dir_from_nuxt_config(cwd))
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
        "imports" | "imports.d.ts" => path.parent()?.to_path_buf(),
        "*" => {
            let parent = path.parent()?;
            if parent.file_name().and_then(|name| name.to_str()) == Some("imports") {
                parent.parent()?.to_path_buf()
            } else {
                parent.to_path_buf()
            }
        }
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
    let source = nuxt_config_source(cwd);
    if source.is_empty() {
        return None;
    }

    let allocator = Allocator::default();
    let source_type = super::parsing::source_type_for_path(Path::new("nuxt.config.ts"));
    let ret = Parser::new(&allocator, source.as_str(), source_type).parse();
    if ret.panicked {
        return None;
    }

    let config_object = default_export_config_object(&ret.program.body)?;
    let build_dir =
        find_object_property(config_object, "buildDir").and_then(static_string_value)?;
    let path = PathBuf::from(build_dir.as_str());
    Some(if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    })
}

fn static_string_value(expression: &Expression<'_>) -> Option<String> {
    match extract_expression(expression)? {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().to_compact_string()),
        Expression::TemplateLiteral(template) => template
            .single_quasi()
            .map(|value| value.as_str().to_compact_string()),
        _ => None,
    }
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

//! Go-to-definition for import and export module specifiers.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde_json::Value;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use vize_carton::cstr;

use super::IdeContext;

#[cfg(test)]
#[path = "module_specifier_tests.rs"]
mod tests;

pub(super) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let specifier = specifier_at_offset(&ctx.content, ctx.offset)?;
    let target = resolve_specifier(ctx.uri, specifier)?;
    let uri = Url::from_file_path(target).ok()?;
    let origin = Position::new(0, 0);

    Some(GotoDefinitionResponse::Scalar(Location {
        uri,
        range: Range::new(origin, origin),
    }))
}

pub(super) fn specifier_at_offset(content: &str, offset: usize) -> Option<&str> {
    if offset > content.len() || !content.is_char_boundary(offset) {
        return None;
    }

    let line_start = content[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[offset..]
        .find('\n')
        .map_or(content.len(), |index| offset + index);
    let line = &content[line_start..line_end];
    let relative_offset = offset - line_start;
    let bytes = line.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let quote = bytes[cursor];
        if quote != b'\'' && quote != b'"' {
            cursor += 1;
            continue;
        }

        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() {
            if bytes[cursor] == b'\\' {
                cursor = (cursor + 2).min(bytes.len());
                continue;
            }
            if bytes[cursor] == quote {
                let end = cursor;
                if relative_offset > start
                    && relative_offset <= end
                    && is_module_context(&line[..start])
                {
                    return Some(&line[start + 1..end]);
                }
                cursor += 1;
                break;
            }
            cursor += 1;
        }
    }

    None
}

fn is_module_context(prefix: &str) -> bool {
    let prefix = prefix.trim_end();
    prefix == "import"
        || prefix.ends_with(" from")
        || prefix.ends_with("import(")
        || prefix.ends_with("require(")
}

pub(super) fn resolve_specifier(current_uri: &Url, specifier: &str) -> Option<PathBuf> {
    let current_file = current_uri.to_file_path().ok()?;
    let current_dir = current_file.parent()?;

    if specifier.starts_with("./") || specifier.starts_with("../") {
        return resolve_file_candidate(&current_dir.join(specifier));
    }
    if specifier.starts_with('/') || specifier.starts_with('#') {
        return None;
    }

    let (package_name, subpath) = split_package_specifier(specifier)?;
    let mut ancestor = Some(current_dir);
    while let Some(dir) = ancestor {
        let package_root = dir.join("node_modules").join(package_name);
        if package_root.join("package.json").is_file() {
            return resolve_package_entry(&package_root, subpath);
        }
        ancestor = dir.parent();
    }
    None
}

fn split_package_specifier(specifier: &str) -> Option<(&str, &str)> {
    if specifier.is_empty() {
        return None;
    }
    if specifier.starts_with('@') {
        let scope_end = specifier.find('/')?;
        let package_end = specifier[scope_end + 1..]
            .find('/')
            .map_or(specifier.len(), |index| scope_end + 1 + index);
        if package_end == scope_end + 1 {
            return None;
        }
        return Some((
            &specifier[..package_end],
            specifier[package_end..].trim_start_matches('/'),
        ));
    }

    let package_end = specifier.find('/').unwrap_or(specifier.len());
    Some((
        &specifier[..package_end],
        specifier[package_end..].trim_start_matches('/'),
    ))
}

fn resolve_package_entry(package_root: &Path, subpath: &str) -> Option<PathBuf> {
    let manifest = fs::read_to_string(package_root.join("package.json")).ok()?;
    let manifest: Value = serde_json::from_str(&manifest).ok()?;

    if let Some(exports) = manifest.get("exports") {
        let export_key = if subpath.is_empty() {
            cstr!(".")
        } else {
            cstr!("./{subpath}")
        };
        let target = select_export(exports, &export_key)?;
        return resolve_package_target(package_root, &target);
    }

    if !subpath.is_empty() {
        return resolve_package_target(package_root, subpath);
    }

    for field in ["types", "typings", "module", "main"] {
        if let Some(target) = manifest.get(field).and_then(Value::as_str)
            && let Some(path) = resolve_package_target(package_root, target)
        {
            return Some(path);
        }
    }
    resolve_file_candidate(&package_root.join("index"))
}

fn select_export(exports: &Value, key: &str) -> Option<String> {
    if let Some(object) = exports.as_object() {
        if let Some(value) = object.get(key) {
            return select_conditional_target(value, None);
        }
        for (pattern, value) in object {
            let Some((prefix, suffix)) = pattern.split_once('*') else {
                continue;
            };
            if let Some(replacement) = key
                .strip_prefix(prefix)
                .and_then(|key| key.strip_suffix(suffix))
            {
                return select_conditional_target(value, Some(replacement));
            }
        }
        if key == "." && !object.keys().any(|key| key.starts_with('.')) {
            return select_conditional_target(exports, None);
        }
        return None;
    }
    (key == ".").then(|| select_conditional_target(exports, None))?
}

fn select_conditional_target(value: &Value, replacement: Option<&str>) -> Option<String> {
    if let Some(target) = value.as_str() {
        return Some(match replacement {
            Some(replacement) => target.replace('*', replacement),
            None => target.to_string(),
        });
    }
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .find_map(|value| select_conditional_target(value, replacement));
    }
    let object = value.as_object()?;
    for condition in ["types", "typings", "import", "default", "node", "require"] {
        if let Some(value) = object.get(condition)
            && let Some(target) = select_conditional_target(value, replacement)
        {
            return Some(target);
        }
    }
    object
        .values()
        .find_map(|value| select_conditional_target(value, replacement))
}

fn resolve_package_target(package_root: &Path, target: &str) -> Option<PathBuf> {
    let relative = target.strip_prefix("./").unwrap_or(target);
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        return None;
    }
    resolve_file_candidate(&package_root.join(relative_path))
}

fn resolve_file_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return candidate.canonicalize().ok();
    }
    for extension in [
        "vue", "d.ts", "d.mts", "d.cts", "ts", "tsx", "mts", "cts", "js", "mjs", "cjs",
    ] {
        let with_extension = candidate.with_extension(extension);
        if with_extension.is_file() {
            return with_extension.canonicalize().ok();
        }
    }
    if candidate.is_dir() {
        for basename in [
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
            "index.ts",
            "index.js",
        ] {
            let index = candidate.join(basename);
            if index.is_file() {
                return index.canonicalize().ok();
            }
        }
    }
    None
}

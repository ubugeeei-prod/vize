use std::path::{Component, Path, PathBuf};

use regex::{Regex, RegexBuilder};
use vize_carton::String;

use super::css::CssAliasRule;
use super::query::split_request;
use super::request::classify_vite_plugin_request;

/// Native split of a Vite module ID into request path and query suffix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViteIdParts {
    pub request: String,
    pub query_suffix: String,
}

/// Split a Vite ID at the first query marker.
pub fn split_id_query(id: &str) -> ViteIdParts {
    let split = split_request(id);
    ViteIdParts {
        request: String::from(split.path),
        query_suffix: String::from(split.query_suffix),
    }
}

/// Returns true when an ID is a bare JS module specifier.
pub fn is_bare_specifier(id: &str) -> bool {
    let split = split_request(id);
    let request = split.path;
    !request.is_empty()
        && !request.starts_with("./")
        && !request.starts_with("../")
        && !request.starts_with('/')
        && !request.starts_with('\0')
        && !request.as_bytes().contains(&b':')
}

/// Normalize an importer into a physical file path suitable for Node's resolver.
pub fn normalize_require_base(importer: Option<&str>) -> Option<String> {
    let importer = importer?;
    let request = classify_vite_plugin_request(importer);
    if let Some(path) = request.vize_virtual_path {
        return Some(path_without_query(path.as_str()));
    }

    if request.is_macro_virtual_id {
        return request
            .stripped_virtual_path
            .map(|path| path_without_query(path.as_str()));
    }

    Some(path_without_query(importer))
}

/// Resolve a Vite alias rule against a request string, preserving query suffixes.
pub fn resolve_alias_request(id: &str, alias_rules: &[CssAliasRule]) -> Option<String> {
    let split = split_request(id);
    for rule in alias_rules {
        if rule.is_regex {
            let pattern = build_alias_regex(rule)?;
            if !pattern.is_match(split.path) {
                continue;
            }
            let replaced = pattern.replace(split.path, rule.replacement.as_str());
            return Some(with_query(replaced.as_ref(), split.query_suffix));
        }

        if split.path == rule.find.as_str() {
            return Some(with_query(rule.replacement.as_str(), split.query_suffix));
        }

        let Some(suffix) = alias_suffix(split.path, rule.find.as_str()) else {
            continue;
        };
        let mut resolved = String::with_capacity(
            rule.replacement.len() + suffix.len() + split.query_suffix.len() + 1,
        );
        resolved.push_str(rule.replacement.as_str());
        if !resolved.ends_with('/') {
            resolved.push('/');
        }
        resolved.push_str(suffix);
        resolved.push_str(split.query_suffix);
        return Some(resolved);
    }

    None
}

/// Normalize Vite-resolved Vue paths into filesystem paths.
pub fn normalize_resolved_vue_path(id: &str) -> Option<String> {
    let split = split_request(id);
    if !split.path.ends_with(".vue") {
        return None;
    }

    Some(String::from(strip_vite_fs_prefix(split.path)))
}

/// Resolve a Vue path against a Vite root/importer pair.
pub fn resolve_vue_path(root: &str, id: &str, importer: Option<&str>) -> String {
    let id_path = Path::new(id);
    let mut resolved = if id.starts_with("/@fs/") {
        PathBuf::from(strip_vite_fs_prefix(id))
    } else if let Some(relative) = id.strip_prefix('/')
        && !id_path.exists()
    {
        Path::new(root).join(relative)
    } else if id_path.is_absolute() {
        id_path.to_path_buf()
    } else if let Some(importer) = importer {
        let importer = real_importer_path(importer);
        importer_parent(importer.as_str()).join(id)
    } else {
        Path::new(root).join(id)
    };

    if !resolved.is_absolute() {
        resolved = Path::new(root).join(resolved);
    }

    path_to_string(&lexical_normalize(&resolved))
}

/// Create package.json bases for resolving bare imports through Node.
pub fn create_bare_import_bases(root: &str, importer: Option<&str>) -> Vec<String> {
    let mut candidates = Vec::with_capacity(4);
    if let Some(base) = normalize_require_base(importer) {
        push_unique(&mut candidates, base);
    }
    push_unique(&mut candidates, path_join_to_string(root, "package.json"));
    push_pnpm_hoist_bases(&mut candidates, importer, false);
    push_pnpm_hoist_bases(&mut candidates, Some(root), true);
    candidates
}

/// Create deduplicated bare import candidates after Vite and alias resolution.
pub fn create_bare_import_candidates(
    id: &str,
    alias_rules: &[CssAliasRule],
    resolved_id: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::with_capacity(3);
    if let Some(resolved_id) = resolved_id
        && is_bare_specifier(resolved_id)
    {
        push_unique(&mut candidates, String::from(resolved_id));
    }
    if let Some(alias_request) = resolve_alias_request(id, alias_rules)
        && is_bare_specifier(alias_request.as_str())
    {
        push_unique(&mut candidates, alias_request);
    }
    if is_bare_specifier(id) {
        push_unique(&mut candidates, String::from(id));
    }
    candidates
}

/// Resolve a relative import with Vite's extension/index fallback order.
pub fn resolve_relative_import(id: &str, importer: &str) -> Option<String> {
    if !id.starts_with("./") && !id.starts_with("../") {
        return None;
    }

    let split = split_request(id);
    let base = lexical_normalize(&importer_parent(importer).join(split.path));
    for extension in ["", ".ts", ".tsx", ".js", ".jsx", ".json"] {
        let candidate = path_with_suffix(&base, extension);
        if candidate.is_file() {
            return Some(with_query(
                path_to_string(&candidate).as_str(),
                split.query_suffix,
            ));
        }
    }

    if base.is_dir() {
        for index_file in ["index.ts", "index.tsx", "index.js", "index.jsx"] {
            let candidate = base.join(index_file);
            if candidate.exists() {
                return Some(with_query(
                    path_to_string(&candidate).as_str(),
                    split.query_suffix,
                ));
            }
        }
    }

    None
}

fn build_alias_regex(rule: &CssAliasRule) -> Option<Regex> {
    let mut builder = RegexBuilder::new(rule.find.as_str());
    if let Some(flags) = rule.flags.as_deref() {
        let bytes = flags.as_bytes();
        builder
            .case_insensitive(bytes.contains(&b'i'))
            .multi_line(bytes.contains(&b'm'))
            .dot_matches_new_line(bytes.contains(&b's'));
    }
    builder.build().ok()
}

fn alias_suffix<'a>(request: &'a str, find: &str) -> Option<&'a str> {
    if find.ends_with('/') {
        return request.strip_prefix(find);
    }

    request
        .strip_prefix(find)
        .and_then(|suffix| suffix.strip_prefix('/'))
}

fn with_query(value: &str, query_suffix: &str) -> String {
    let mut output = String::with_capacity(value.len() + query_suffix.len());
    output.push_str(value);
    output.push_str(query_suffix);
    output
}

fn path_without_query(value: &str) -> String {
    String::from(split_request(value).path)
}

fn strip_vite_fs_prefix(path: &str) -> &str {
    path.strip_prefix("/@fs")
        .filter(|rest| rest.starts_with('/'))
        .unwrap_or(path)
}

fn real_importer_path(importer: &str) -> String {
    let request = classify_vite_plugin_request(importer);
    if let Some(path) = request.vize_virtual_path {
        return path;
    }
    request
        .stripped_virtual_path
        .unwrap_or_else(|| String::from(importer))
}

fn importer_parent(importer: &str) -> PathBuf {
    Path::new(importer)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn path_join_to_string(base: &str, tail: &str) -> String {
    path_to_string(&Path::new(base).join(tail))
}

fn push_pnpm_hoist_bases(candidates: &mut Vec<String>, start: Option<&str>, is_directory: bool) {
    let Some(start) = start else {
        return;
    };

    let mut dir = if is_directory {
        PathBuf::from(start)
    } else {
        importer_parent(start)
    };

    loop {
        let pnpm_hoist = dir.join("node_modules").join(".pnpm").join("node_modules");
        if pnpm_hoist.exists() {
            push_unique(candidates, path_to_string(&pnpm_hoist.join("package.json")));
            break;
        }

        if !dir.pop() {
            break;
        }
    }
}

fn push_unique(candidates: &mut Vec<String>, candidate: String) {
    if !candidates
        .iter()
        .any(|existing| existing.as_str() == candidate.as_str())
    {
        candidates.push(candidate);
    }
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    if suffix.is_empty() {
        return path.to_path_buf();
    }

    let mut value = path_to_string(path);
    value.push_str(suffix);
    PathBuf::from(value.as_str())
}

fn path_to_string(path: &Path) -> String {
    let value = path.to_string_lossy();
    String::from(value.as_ref())
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    let mut normal_count = 0usize;

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normal_count > 0 {
                    normal_count -= 1;
                    normalized.pop();
                } else if !normalized.is_absolute() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(part) => {
                normal_count += 1;
                normalized.push(part);
            }
        }
    }

    normalized
}

#[cfg(test)]
#[expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
mod tests;

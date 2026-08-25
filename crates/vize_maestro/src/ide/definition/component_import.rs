//! Component import resolution beyond direct `.vue` specifiers.
#![allow(clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::Url;

use super::{IdeContext, helpers};

const MAX_REEXPORT_DEPTH: usize = 8;

pub(crate) fn resolve_component_file(
    ctx: &IdeContext<'_>,
    component_name: &str,
) -> Option<PathBuf> {
    let import = find_component_import(ctx, component_name)?;
    let resolved = helpers::resolve_import_path(ctx.uri, &import.specifier)?;
    if is_vue_path(&resolved) {
        return Some(resolved);
    }
    resolve_exported_component(&resolved, &import.imported_name, 0)
}

struct ComponentImport {
    specifier: String,
    imported_name: String,
}

fn find_component_import(ctx: &IdeContext<'_>, component_name: &str) -> Option<ComponentImport> {
    for name in crate::ide::component_name_candidates(component_name) {
        if let Some(component_import) = find_component_import_by_name(ctx, &name) {
            return Some(component_import);
        }
    }
    None
}

fn find_component_import_by_name(
    ctx: &IdeContext<'_>,
    component_name: &str,
) -> Option<ComponentImport> {
    for (pos, _) in ctx.content.match_indices("import ") {
        let rest = &ctx.content[pos..];
        let Some(from_pos) = rest.find(" from") else {
            continue;
        };
        let specifier = helpers::extract_import_path_from_pos(rest, from_pos + " from".len())?;
        let clause = rest["import ".len()..from_pos].trim();
        if default_import_name(clause) == Some(component_name) {
            return Some(ComponentImport {
                specifier,
                imported_name: "default".to_string(),
            });
        }
        if let Some(imported_name) = named_import_name(clause, component_name) {
            return Some(ComponentImport {
                specifier,
                imported_name,
            });
        }
    }
    None
}

fn default_import_name(clause: &str) -> Option<&str> {
    let clause = clause.strip_prefix("type ").unwrap_or(clause).trim();
    if clause.starts_with('{') || clause.starts_with('*') {
        return None;
    }
    ident_at_start(
        clause
            .split_once(',')
            .map_or(clause, |(head, _)| head)
            .trim(),
    )
}

fn named_import_name(clause: &str, local_name: &str) -> Option<String> {
    let body = brace_body(clause)?;
    for part in body.split(',') {
        let (imported, local) = parse_import_binding(part)?;
        if local == local_name {
            return Some(imported.to_string());
        }
    }
    None
}

fn parse_import_binding(part: &str) -> Option<(&str, &str)> {
    let part = part
        .trim()
        .strip_prefix("type ")
        .unwrap_or(part.trim())
        .trim();
    if part.is_empty() {
        return None;
    }
    if let Some((imported, local)) = part.split_once(" as ") {
        return Some((
            ident_at_start(imported.trim())?,
            ident_at_start(local.trim())?,
        ));
    }
    let name = ident_at_start(part)?;
    Some((name, name))
}

fn resolve_exported_component(
    module_path: &Path,
    export_name: &str,
    depth: usize,
) -> Option<PathBuf> {
    if depth >= MAX_REEXPORT_DEPTH {
        return None;
    }
    let content = std::fs::read_to_string(module_path).ok()?;
    for export in named_reexports(&content, export_name) {
        if let Some(resolved) =
            resolve_reexport(module_path, &export.specifier, &export.imported, depth)
        {
            return Some(resolved);
        }
    }
    for specifier in star_reexports(&content) {
        let Some(target) = resolve_from_module(module_path, &specifier) else {
            continue;
        };
        if is_vue_path(&target) {
            continue;
        }
        if let Some(resolved) = resolve_exported_component(&target, export_name, depth + 1) {
            return Some(resolved);
        }
    }
    None
}

struct Reexport {
    imported: String,
    specifier: String,
}

fn named_reexports(content: &str, export_name: &str) -> Vec<Reexport> {
    let mut exports = Vec::new();
    for (pos, _) in content.match_indices("export ") {
        let rest = &content[pos..];
        let Some(body) = brace_body(rest) else {
            continue;
        };
        let Some(body_start) = rest.find('{') else {
            continue;
        };
        let after_body = &rest[body_start + body.len() + 2..];
        let Some(from_pos) = after_body.find("from") else {
            continue;
        };
        let Some(specifier) =
            helpers::extract_import_path_from_pos(after_body, from_pos + "from".len())
        else {
            continue;
        };
        for part in body.split(',') {
            let Some((imported, exported)) = parse_export_binding(part) else {
                continue;
            };
            if exported == export_name {
                exports.push(Reexport {
                    imported: imported.to_string(),
                    specifier: specifier.clone(),
                });
            }
        }
    }
    exports
}

fn parse_export_binding(part: &str) -> Option<(&str, &str)> {
    let part = part
        .trim()
        .strip_prefix("type ")
        .unwrap_or(part.trim())
        .trim();
    if part.is_empty() {
        return None;
    }
    if let Some((imported, exported)) = part.split_once(" as ") {
        return Some((
            ident_at_start(imported.trim())?,
            ident_at_start(exported.trim())?,
        ));
    }
    let name = ident_at_start(part)?;
    Some((name, name))
}

fn star_reexports(content: &str) -> Vec<String> {
    let mut specifiers = Vec::new();
    for (pos, _) in content.match_indices("export *") {
        let rest = &content[pos + "export *".len()..];
        let Some(from_pos) = rest.find("from") else {
            continue;
        };
        if let Some(specifier) =
            helpers::extract_import_path_from_pos(rest, from_pos + "from".len())
        {
            specifiers.push(specifier);
        }
    }
    specifiers
}

fn resolve_reexport(
    module_path: &Path,
    specifier: &str,
    imported: &str,
    depth: usize,
) -> Option<PathBuf> {
    let target = resolve_from_module(module_path, specifier)?;
    if is_vue_path(&target) {
        return (imported == "default").then_some(target);
    }
    resolve_exported_component(&target, imported, depth + 1)
}

fn resolve_from_module(module_path: &Path, specifier: &str) -> Option<PathBuf> {
    let uri = Url::from_file_path(module_path).ok()?;
    helpers::resolve_import_path(&uri, specifier)
}

fn brace_body(value: &str) -> Option<&str> {
    let start = value.find('{')?;
    let end = value[start + 1..].find('}')? + start + 1;
    Some(&value[start + 1..end])
}

fn ident_at_start(value: &str) -> Option<&str> {
    let end = value
        .bytes()
        .position(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'))
        .unwrap_or(value.len());
    (end > 0).then_some(&value[..end])
}

fn is_vue_path(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tower_lsp::lsp_types::Url;

    use super::*;
    use crate::server::ServerState;

    #[test]
    fn resolve_component_file_prefers_exact_binding_over_lower_camel_fallback() {
        let workspace = tempfile::tempdir().expect("temporary workspace");
        let lower_camel_path = workspace.path().join("lower.vue");
        let exact_path = workspace.path().join("exact.vue");
        fs::write(&lower_camel_path, "<template><span /></template>\n").expect("lower component");
        fs::write(&exact_path, "<template><strong /></template>\n").expect("exact component");

        let importer_path = workspace.path().join("App.vue");
        let source = r#"<script setup lang="ts">
import descriptionItem from "./lower.vue";
import DescriptionItem from "./exact.vue";
</script>
<template><DescriptionItem /></template>
"#;
        fs::write(&importer_path, source).expect("importer");

        let state = ServerState::new();
        let uri = Url::from_file_path(importer_path).expect("importer URI");
        let offset = source.rfind("DescriptionItem").expect("component tag");
        let ctx = IdeContext::with_content(&state, &uri, offset, source.to_owned());
        let resolved = resolve_component_file(&ctx, "DescriptionItem").expect("component file");

        assert_eq!(
            resolved.canonicalize().expect("canonical resolved path"),
            exact_path.canonicalize().expect("canonical exact path")
        );
    }
}

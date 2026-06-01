//! Void Vue route-target editor helpers.

use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use vize_carton::{FxHashSet, String, cstr};

use super::context::{is_ident_byte, string_literal_at_cursor};
use crate::ide::IdeContext;

const ROUTE_COMPLETION_LIMIT: usize = 200;
const VOID_NAVIGATION_CALLS: &[&str] = &["useForm", "action", "visit", "prefetch"];

pub(crate) fn completions(ctx: &IdeContext<'_>) -> Vec<CompletionItem> {
    if !imports_void_vue(&ctx.content) || !is_void_route_context(&ctx.content, ctx.offset) {
        return Vec::new();
    }

    collect_page_routes(ctx.uri)
        .into_iter()
        .map(|route| CompletionItem {
            label: route.to_string(),
            kind: Some(CompletionItemKind::FILE),
            detail: Some(std::string::String::from("Void page route")),
            sort_text: Some(std::string::String::from("0")),
            ..Default::default()
        })
        .collect()
}

fn is_void_route_context(content: &str, offset: usize) -> bool {
    let Some(ctx) = string_literal_at_cursor(content, offset) else {
        return false;
    };

    is_link_href_context(content, ctx.open) || is_void_navigation_call(ctx.before_open)
}

fn is_link_href_context(content: &str, offset: usize) -> bool {
    let before_open = &content[..offset];
    let before_attr = before_open.trim_end();
    let is_href_value = before_attr.ends_with("href=")
        || before_attr.ends_with(":href=")
        || before_attr.ends_with("v-bind:href=");
    if !is_href_value {
        return false;
    }

    let Some(tag_start) = find_link_tag_start(before_open) else {
        return false;
    };

    !before_open[tag_start..].contains('>')
}

fn find_link_tag_start(before: &str) -> Option<usize> {
    let mut cursor = before.len();
    while let Some(found) = before[..cursor].rfind("<Link") {
        let after_tag = found + "<Link".len();
        let boundary = before
            .as_bytes()
            .get(after_tag)
            .map(|byte| byte.is_ascii_whitespace() || *byte == b'/' || *byte == b'>')
            .unwrap_or(true);
        if boundary {
            return Some(found);
        }
        cursor = found;
    }

    None
}

fn is_void_navigation_call(before_open: &str) -> bool {
    let before_call = before_open.trim_end();
    let Some(before_call) = before_call.strip_suffix('(') else {
        return false;
    };
    let before_call = before_call.trim_end();

    VOID_NAVIGATION_CALLS
        .iter()
        .any(|name| ends_with_call_name(before_call, name))
}

fn ends_with_call_name(text: &str, name: &str) -> bool {
    let Some(prefix) = text.strip_suffix(name) else {
        return false;
    };
    prefix
        .as_bytes()
        .last()
        .map(|byte| !is_ident_byte(*byte))
        .unwrap_or(true)
}

fn imports_void_vue(source: &str) -> bool {
    source.contains("from \"@void/vue\"")
        || source.contains("from '@void/vue'")
        || source.contains("from \"@void/vue/client\"")
        || source.contains("from '@void/vue/client'")
}

fn collect_page_routes(uri: &tower_lsp::lsp_types::Url) -> Vec<String> {
    let Ok(file_path) = uri.to_file_path() else {
        return Vec::new();
    };
    let Some(start_dir) = file_path.parent() else {
        return Vec::new();
    };
    let Some(pages_dir) = find_pages_dir(start_dir) else {
        return Vec::new();
    };

    let mut seen = FxHashSet::<String>::default();
    let mut routes = Vec::new();
    collect_routes_from_dir(&pages_dir, &pages_dir, &mut seen, &mut routes);
    routes.sort();
    routes
}

fn find_pages_dir(start_dir: &Path) -> Option<PathBuf> {
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        if dir.file_name().and_then(|name| name.to_str()) == Some("pages") {
            return Some(dir.to_path_buf());
        }

        let candidate = dir.join("pages");
        if candidate.is_dir() {
            return Some(candidate);
        }

        current = dir.parent();
    }

    None
}

fn collect_routes_from_dir(
    root: &Path,
    dir: &Path,
    seen: &mut FxHashSet<String>,
    routes: &mut Vec<String>,
) {
    if routes.len() >= ROUTE_COMPLETION_LIMIT {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_skipped_dir(&path) {
                continue;
            }
            collect_routes_from_dir(root, &path, seen, routes);
            continue;
        }

        let Some(route) = route_from_page_file(root, &path) else {
            continue;
        };
        if seen.insert(route.clone()) {
            routes.push(route);
        }
        if routes.len() >= ROUTE_COMPLETION_LIMIT {
            return;
        }
    }
}

fn is_skipped_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("node_modules" | ".git" | ".void" | "dist" | "build")
    )
}

fn route_from_page_file(root: &Path, path: &Path) -> Option<String> {
    if !is_page_route_file(path) {
        return None;
    }

    let relative = path.strip_prefix(root).ok()?;
    let mut parts: Vec<String> = relative
        .iter()
        .filter_map(|part| part.to_str())
        .map(String::from)
        .collect();
    let filename = parts.pop()?;
    let stem = page_stem(&filename)?;
    if stem != "index" {
        parts.push(stem);
    }

    let segments = parts
        .into_iter()
        .filter(|part| part != "index")
        .filter_map(|part| route_segment(&part))
        .collect::<Vec<_>>();

    if segments.is_empty() {
        Some(String::from("/"))
    } else {
        let path = segments.join("/");
        Some(cstr!("/{path}"))
    }
}

fn is_page_route_file(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if filename.starts_with('_') || filename.starts_with('.') {
        return false;
    }

    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("vue" | "ts" | "tsx" | "js" | "jsx")
    )
}

fn page_stem(filename: &str) -> Option<String> {
    let stem = filename
        .strip_suffix(".vue")
        .or_else(|| filename.strip_suffix(".tsx"))
        .or_else(|| filename.strip_suffix(".ts"))
        .or_else(|| filename.strip_suffix(".jsx"))
        .or_else(|| filename.strip_suffix(".js"))?;

    let stem = stem.strip_suffix(".server").unwrap_or(stem);
    if matches!(stem, "layout" | "middleware" | "error") {
        return None;
    }
    Some(String::from(stem))
}

fn route_segment(segment: &str) -> Option<String> {
    if matches!(segment, "layout" | "middleware" | "error") {
        return None;
    }

    if let Some(name) = segment
        .strip_prefix("[[...")
        .and_then(|value| value.strip_suffix("]]"))
        .or_else(|| {
            segment
                .strip_prefix("[...")
                .and_then(|value| value.strip_suffix(']'))
        })
    {
        return (!name.is_empty()).then(|| cstr!(":{name}*"));
    }

    if let Some(name) = segment
        .strip_prefix("[[")
        .and_then(|value| value.strip_suffix("]]"))
    {
        return (!name.is_empty()).then(|| cstr!(":{name}?"));
    }

    if let Some(name) = segment
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        return (!name.is_empty()).then(|| cstr!(":{name}"));
    }

    Some(String::from(segment))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_void_route_context, route_from_page_file};

    #[test]
    fn detects_void_link_and_use_form_route_contexts() {
        let template = r#"<Link href="/settings">Settings</Link>"#;
        assert!(is_void_route_context(
            template,
            template.find("settings").unwrap()
        ));

        let script = r#"const form = useForm("/posts", {})"#;
        assert!(is_void_route_context(script, script.find("posts").unwrap()));
    }

    #[test]
    fn maps_void_page_files_to_route_paths() {
        let root = Path::new("/app/pages");

        assert_eq!(
            route_from_page_file(root, Path::new("/app/pages/index.vue")).as_deref(),
            Some("/")
        );
        assert_eq!(
            route_from_page_file(root, Path::new("/app/pages/users/[id].vue")).as_deref(),
            Some("/users/:id")
        );
        assert_eq!(
            route_from_page_file(root, Path::new("/app/pages/docs/[...slug].server.ts")).as_deref(),
            Some("/docs/:slug*")
        );
    }
}

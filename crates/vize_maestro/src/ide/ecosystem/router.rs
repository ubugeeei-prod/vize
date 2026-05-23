//! Vue Router route-name and file-route param editor helpers.

use std::path::{Path, PathBuf};

use serde_json::Value;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Diagnostic, Position, Range, Url};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{FxHashSet, String, cstr};

use super::context::{preceding_property_is_name, string_literal_at_cursor};
use crate::ide::{IdeContext, offset_to_position};

const ROUTER_PUSH: &str = "router.push(";
const ROUTER_REPLACE: &str = "router.replace(";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteParam {
    name: String,
    optional: bool,
    repeatable: bool,
}

pub(crate) fn completions(
    ctx: &IdeContext<'_>,
    descriptor: &SfcDescriptor<'_>,
) -> Vec<CompletionItem> {
    if let Some(items) = route_param_completions(ctx) {
        return items;
    }

    if !is_route_name_context(&ctx.content, ctx.offset) {
        return Vec::new();
    }

    route_names(descriptor)
        .into_iter()
        .map(|name| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(std::string::String::from("Vue Router route name")),
            sort_text: Some(std::string::String::from("0")),
            ..Default::default()
        })
        .collect()
}

pub(crate) fn route_names(descriptor: &SfcDescriptor<'_>) -> Vec<String> {
    let mut seen = FxHashSet::<String>::default();
    let mut names = Vec::new();

    for block in &descriptor.custom_blocks {
        if block.block_type != "route" || !is_json_lang(block) {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&block.content) else {
            continue;
        };
        if let Some(name) = value.get("name").and_then(Value::as_str) {
            push_name(name, &mut seen, &mut names);
        }
    }

    if let Some(script) = &descriptor.script {
        collect_define_page_names(&script.content, &mut seen, &mut names);
    }
    if let Some(script_setup) = &descriptor.script_setup {
        collect_define_page_names(&script_setup.content, &mut seen, &mut names);
    }

    names
}

pub(crate) fn route_param_diagnostics(content: &str, uri: &Url) -> Vec<Diagnostic> {
    let params = route_params_for_file(uri);
    if params.is_empty() && !is_page_file(uri) {
        return Vec::new();
    }

    let allowed: FxHashSet<String> = params.iter().map(|param| param.name.clone()).collect();
    let route_identifiers = route_identifiers(content);
    let mut diagnostics = Vec::new();
    let mut pos = 0usize;

    while let Some(found) = content[pos..].find(".params.") {
        let dot = pos + found;
        let Some(object_name) = object_name_before_dot(content, dot) else {
            pos = dot + ".params.".len();
            continue;
        };
        if object_name != "$route" && !route_identifiers.contains(object_name) {
            pos = dot + ".params.".len();
            continue;
        }

        let name_start = dot + ".params.".len();
        let Some((name, name_end)) = identifier_at(content, name_start) else {
            pos = name_start;
            continue;
        };
        if !allowed.contains(name) {
            diagnostics.push(super::warning_diagnostic(
                offset_range(content, name_start, name_end),
                "ecosystem/vue-router-route-param",
                unknown_param_message(name, &params),
            ));
        }
        pos = name_end;
    }

    diagnostics
}

fn route_param_completions(ctx: &IdeContext<'_>) -> Option<Vec<CompletionItem>> {
    if !is_route_param_completion_context(&ctx.content, ctx.offset) {
        return None;
    }

    let params = route_params_for_file(ctx.uri);
    if params.is_empty() {
        return None;
    }

    Some(
        params
            .into_iter()
            .map(|param| CompletionItem {
                label: param.name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(route_param_detail(&param)),
                sort_text: Some(std::string::String::from("0")),
                ..Default::default()
            })
            .collect(),
    )
}

fn is_route_param_completion_context(content: &str, offset: usize) -> bool {
    let before = &content[..offset.min(content.len())];
    let Some(params_pos) = before.rfind(".params.") else {
        return false;
    };
    if before[params_pos + ".params.".len()..]
        .bytes()
        .any(|byte| !super::context::is_ident_byte(byte))
    {
        return false;
    }

    let Some(object_name) = object_name_before_dot(content, params_pos) else {
        return false;
    };
    object_name == "$route" || route_identifiers(content).contains(object_name)
}

fn route_params_for_file(uri: &Url) -> Vec<RouteParam> {
    let Ok(file_path) = uri.to_file_path() else {
        return Vec::new();
    };
    let Some(pages_dir) = find_pages_dir_for_file(&file_path) else {
        return Vec::new();
    };
    let Ok(relative) = file_path.strip_prefix(&pages_dir) else {
        return Vec::new();
    };

    let mut seen = FxHashSet::<String>::default();
    let mut params = Vec::new();
    for part in relative.iter().filter_map(|part| part.to_str()) {
        let part = page_segment_stem(part).unwrap_or(part);
        collect_segment_params(part, &mut seen, &mut params);
    }
    params
}

fn is_page_file(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|path| find_pages_dir_for_file(&path))
        .is_some()
}

fn find_pages_dir_for_file(file_path: &Path) -> Option<PathBuf> {
    let mut current = file_path.parent();
    while let Some(dir) = current {
        if dir.file_name().and_then(|name| name.to_str()) == Some("pages") {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn page_segment_stem(segment: &str) -> Option<&str> {
    segment
        .strip_suffix(".vue")
        .or_else(|| segment.strip_suffix(".tsx"))
        .or_else(|| segment.strip_suffix(".ts"))
        .or_else(|| segment.strip_suffix(".jsx"))
        .or_else(|| segment.strip_suffix(".js"))
        .map(|stem| stem.split('@').next().unwrap_or(stem))
}

fn collect_segment_params(
    segment: &str,
    seen: &mut FxHashSet<String>,
    params: &mut Vec<RouteParam>,
) {
    let mut cursor = 0usize;
    while let Some(open_rel) = segment[cursor..].find('[') {
        let open = cursor + open_rel;
        let optional = segment[open..].starts_with("[[");
        let value_start = open + if optional { 2 } else { 1 };
        let close_marker = if optional { "]]" } else { "]" };
        let Some(close_rel) = segment[value_start..].find(close_marker) else {
            break;
        };
        let close = value_start + close_rel;
        let repeatable = segment[close + close_marker.len()..].starts_with('+');
        let mut name = &segment[value_start..close];
        let catch_all = name.starts_with("...");
        if catch_all {
            name = &name[3..];
        }

        if !name.is_empty() {
            let name = String::from(name);
            if seen.insert(name.clone()) {
                params.push(RouteParam {
                    name,
                    optional,
                    repeatable: repeatable || catch_all,
                });
            }
        }
        cursor = close + close_marker.len() + usize::from(repeatable);
    }
}

fn route_identifiers(content: &str) -> FxHashSet<&str> {
    let mut identifiers = FxHashSet::default();
    identifiers.insert("$route");

    let mut pos = 0usize;
    while let Some(found) = content[pos..].find("useRoute") {
        let call_start = pos + found;
        let after_name = call_start + "useRoute".len();
        let Some(next) = content.as_bytes().get(after_name).copied() else {
            break;
        };
        if next != b'(' && next != b'<' {
            pos = after_name;
            continue;
        }
        if let Some(identifier) = assigned_identifier_before(content, call_start) {
            identifiers.insert(identifier);
        }
        pos = after_name;
    }

    identifiers
}

fn assigned_identifier_before(content: &str, call_start: usize) -> Option<&str> {
    let window_start = call_start.saturating_sub(160);
    let before_call = &content[window_start..call_start];
    let eq = before_call.rfind('=')?;
    let before_eq = before_call[..eq].trim_end();
    let end = before_eq.len();
    let start = before_eq[..end]
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let identifier = &before_eq[start..end];
    (!identifier.is_empty()).then_some(identifier)
}

fn object_name_before_dot(content: &str, dot: usize) -> Option<&str> {
    let bytes = content.as_bytes();
    let mut start = dot;
    while start > 0 {
        let byte = bytes[start - 1];
        if super::context::is_ident_byte(byte) || byte == b'$' {
            start -= 1;
        } else {
            break;
        }
    }
    (start < dot).then_some(&content[start..dot])
}

fn identifier_at(content: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    let first = *bytes.get(start)?;
    if !super::context::is_ident_byte(first) {
        return None;
    }
    let mut end = start + 1;
    while bytes
        .get(end)
        .is_some_and(|byte| super::context::is_ident_byte(*byte))
    {
        end += 1;
    }
    Some((&content[start..end], end))
}

fn offset_range(content: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = offset_to_position(content, start);
    let (end_line, end_character) = offset_to_position(content, end);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

fn route_param_detail(param: &RouteParam) -> std::string::String {
    let ty = match (param.optional, param.repeatable) {
        (true, true) => "string[] | undefined",
        (true, false) => "string | undefined",
        (false, true) => "string[]",
        (false, false) => "string",
    };
    cstr!("Vue Router file route param: {ty}").to_string()
}

fn unknown_param_message(name: &str, params: &[RouteParam]) -> std::string::String {
    if params.is_empty() {
        return cstr!("Route param `{name}` is not defined by this page file").to_string();
    }

    let available = params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    cstr!("Route param `{name}` is not defined by this page file. Available params: {available}")
        .to_string()
}

fn is_route_name_context(content: &str, offset: usize) -> bool {
    let Some(ctx) = string_literal_at_cursor(content, offset) else {
        return false;
    };
    if !preceding_property_is_name(ctx.before_open) {
        return false;
    }

    let search_start = ctx.open.saturating_sub(512);
    let nearby = &content[search_start..ctx.open];
    nearby.contains(ROUTER_PUSH)
        || nearby.contains(ROUTER_REPLACE)
        || is_router_link_to_binding(content, ctx.open)
}

fn is_router_link_to_binding(content: &str, offset: usize) -> bool {
    let before = &content[..offset];
    let Some(tag_start) = find_router_link_tag_start(before) else {
        return false;
    };
    if before[tag_start..].contains('>') {
        return false;
    }
    let tag = &before[tag_start..];
    tag.contains(":to=") || tag.contains("v-bind:to=")
}

fn find_router_link_tag_start(before: &str) -> Option<usize> {
    ["<RouterLink", "<router-link", "<NuxtLink", "<nuxt-link"]
        .iter()
        .filter_map(|tag| before.rfind(tag))
        .max()
}

fn is_json_lang(block: &vize_atelier_sfc::SfcCustomBlock<'_>) -> bool {
    block
        .attrs
        .get("lang")
        .map(|lang| lang.as_ref() == "json")
        .unwrap_or(true)
}

fn collect_define_page_names(source: &str, seen: &mut FxHashSet<String>, names: &mut Vec<String>) {
    let mut pos = 0usize;
    while let Some(found) = source[pos..].find("definePage") {
        let call_start = pos + found;
        let window_end = (call_start + 512).min(source.len());
        if let Some(name) = property_string(&source[call_start..window_end], "name") {
            push_name(name, seen, names);
        }
        pos = call_start + "definePage".len();
    }
}

fn property_string<'a>(source: &'a str, property: &str) -> Option<&'a str> {
    let mut pos = 0usize;
    while let Some(found) = source[pos..].find(property) {
        let key_start = pos + found;
        let before_ok = key_start == 0
            || source
                .as_bytes()
                .get(key_start - 1)
                .is_some_and(|byte| !super::context::is_ident_byte(*byte));
        let key_end = key_start + property.len();
        let after_ok = source
            .as_bytes()
            .get(key_end)
            .is_some_and(|byte| !super::context::is_ident_byte(*byte));
        if !before_ok || !after_ok {
            pos = key_end;
            continue;
        }

        let mut cursor = key_end;
        skip_ascii_ws(source.as_bytes(), &mut cursor);
        if source.as_bytes().get(cursor).copied() != Some(b':') {
            pos = key_end;
            continue;
        }
        cursor += 1;
        skip_ascii_ws(source.as_bytes(), &mut cursor);
        let quote = source.as_bytes().get(cursor).copied()?;
        if quote != b'\'' && quote != b'"' {
            pos = cursor + 1;
            continue;
        }
        let value_start = cursor + 1;
        let mut value_end = value_start;
        while value_end < source.len() {
            if source.as_bytes()[value_end] == quote {
                return Some(&source[value_start..value_end]);
            }
            value_end += 1;
        }
        return None;
    }

    None
}

fn skip_ascii_ws(bytes: &[u8], pos: &mut usize) {
    while bytes.get(*pos).is_some_and(u8::is_ascii_whitespace) {
        *pos += 1;
    }
}

fn push_name(name: &str, seen: &mut FxHashSet<String>, names: &mut Vec<String>) {
    if name.is_empty() {
        return;
    }
    let name = String::from(name);
    if seen.insert(name.clone()) {
        names.push(name);
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::{is_route_name_context, route_names, route_params_for_file};

    #[test]
    fn detects_router_push_and_router_link_name_contexts() {
        let script = r#"router.push({ name: "dash" })"#;
        assert!(is_route_name_context(script, script.find("dash").unwrap()));

        let template = r#"<RouterLink :to="{ name: 'dash' }" />"#;
        assert!(is_route_name_context(
            template,
            template.find("dash").unwrap()
        ));
    }

    #[test]
    fn extracts_route_names_from_route_block_and_define_page() {
        let source = r#"<script setup>
definePage({ name: "settings" })
</script>
<route lang="json">
{ "name": "home" }
</route>
"#;
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let names = route_names(&descriptor);

        assert_eq!(names, vec!["home", "settings"]);
    }

    #[test]
    fn infers_params_from_vue_router_file_names() {
        let uri = Url::parse("file:///repo/src/pages/articles/[[slug]]+/product_[skuId]_[seo].vue")
            .unwrap();
        let params = route_params_for_file(&uri);
        let names = params
            .iter()
            .map(|param| (param.name.as_str(), param.optional, param.repeatable))
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                ("slug", true, true),
                ("skuId", false, false),
                ("seo", false, false)
            ]
        );
    }
}

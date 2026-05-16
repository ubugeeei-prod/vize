//! Same-file Vue Router 5 route-name editor helpers.

use serde_json::Value;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{FxHashSet, String};

use super::context::{preceding_property_is_name, string_literal_at_cursor};

const ROUTER_PUSH: &str = "router.push(";
const ROUTER_REPLACE: &str = "router.replace(";

pub(crate) fn completions(
    content: &str,
    offset: usize,
    descriptor: &SfcDescriptor<'_>,
) -> Vec<CompletionItem> {
    if !is_route_name_context(content, offset) {
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
    use super::{is_route_name_context, route_names};

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
}

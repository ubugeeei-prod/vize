//! Imported-component metadata, caching, and prop/slot completion items.

use std::{collections::BTreeSet, sync::Arc};

use oxc_ast::ast::{PropertyKey, Statement, TSSignature, TSType};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind,
};
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use crate::ide::definition::helpers as definition_helpers;
use crate::ide::{IdeContext, is_component_tag, kebab_to_pascal, pascal_to_kebab};

use super::tag_context::{
    find_attr_value, find_tag_end, is_dynamic_prop_prefix, is_prop_completion_prefix,
    is_slot_completion_prefix, nearest_open_component_before, opening_tag_context_at_offset,
};

pub(crate) fn component_surface_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let Some(tag_ctx) = opening_tag_context_at_offset(&ctx.content, ctx.offset) else {
        return Vec::new();
    };

    if tag_ctx.inside_attribute_value {
        return Vec::new();
    }

    if tag_ctx.tag_name == "template" {
        if !is_slot_completion_prefix(&tag_ctx.current_token) {
            return Vec::new();
        }

        let Some(component_name) = nearest_open_component_before(&ctx.content, tag_ctx.tag_start)
        else {
            return Vec::new();
        };
        let Some(metadata) = component_metadata(ctx, &component_name) else {
            return Vec::new();
        };

        return metadata
            .slots
            .iter()
            .map(|slot| slot_completion_item(slot, &tag_ctx.current_token))
            .collect();
    }

    if !is_component_tag(&tag_ctx.tag_name) || !is_prop_completion_prefix(&tag_ctx.current_token) {
        return Vec::new();
    }

    let Some(metadata) = component_metadata(ctx, &tag_ctx.tag_name) else {
        return Vec::new();
    };
    let dynamic = is_dynamic_prop_prefix(&tag_ctx.current_token);

    metadata
        .props
        .iter()
        .map(|prop| prop_completion_item(prop, dynamic))
        .collect()
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentMetadata {
    props: Vec<ComponentProp>,
    slots: Vec<ComponentSlot>,
}

/// Cached metadata for an imported component, keyed by resolved path.
/// `len` + `modified` invalidate the entry when the file changes on disk.
#[derive(Clone)]
pub(crate) struct CachedComponentMetadata {
    pub len: u64,
    pub modified: Option<std::time::SystemTime>,
    pub metadata: Arc<ComponentMetadata>,
}

#[derive(Debug, Clone)]
struct ComponentProp {
    name: String,
    type_detail: Option<String>,
    required: bool,
    /// Default value source — populated from `withDefaults` or per-prop
    /// `default` config. Renders into the completion documentation so the
    /// user knows what the prop falls back to.
    default_value: Option<String>,
}

#[derive(Debug, Clone)]
struct ComponentSlot {
    name: String,
    props_type: Option<String>,
}

fn component_metadata(ctx: &IdeContext, component_name: &str) -> Option<Arc<ComponentMetadata>> {
    if let Some(metadata) = super::self_component::metadata(ctx, component_name) {
        return Some(metadata);
    }

    let mut names = vec![component_name.to_string()];
    let pascal = kebab_to_pascal(component_name);
    if !names.iter().any(|name| name == &pascal) {
        names.push(pascal);
    }

    for name in names {
        let Some(import_path) = definition_helpers::find_import_path(ctx, &name) else {
            continue;
        };
        let resolved = definition_helpers::resolve_import_path(ctx.uri, &import_path)?;
        return cached_component_metadata(ctx, &resolved);
    }

    if let Some(import_path) = art_component_path(ctx, component_name) {
        let resolved = definition_helpers::resolve_import_path(ctx.uri, &import_path)?;
        return cached_component_metadata(ctx, &resolved);
    }

    None
}

/// Return parsed metadata for the component at `resolved`, reusing a cached
/// parse when the file's length + modification time are unchanged. Only the
/// `fs::metadata` stat runs on the hot (cache-hit) path; the disk read, SFC
/// parse, and Croquis analysis happen solely on a miss.
pub(super) fn cached_component_metadata(
    ctx: &IdeContext,
    resolved: &std::path::Path,
) -> Option<Arc<ComponentMetadata>> {
    let cache = ctx.state.component_metadata_cache();
    let (len, modified) = std::fs::metadata(resolved)
        .map(|meta| (meta.len(), meta.modified().ok()))
        .unwrap_or((0, None));

    if let Some(entry) = cache.get(resolved)
        && entry.len == len
        && entry.modified == modified
    {
        return Some(entry.metadata.clone());
    }

    let component_content = std::fs::read_to_string(resolved).ok()?;
    let metadata = Arc::new(extract_component_metadata(
        &component_content,
        &resolved.to_string_lossy(),
        ctx.state.options_api_enabled(),
        ctx.state.legacy_vue2_enabled(),
    ));
    cache.insert(
        resolved.to_path_buf(),
        CachedComponentMetadata {
            len,
            modified,
            metadata: metadata.clone(),
        },
    );
    Some(metadata)
}

fn art_component_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    if !ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let allocator = vize_carton::Bump::new();
    let art_desc = vize_musea::parse_art(
        &allocator,
        &ctx.content,
        vize_musea::ArtParseOptions::default(),
    )
    .ok()?;
    let component_path = art_desc.metadata.component?;
    // Reuse the script_setup already extracted by `parse_art` above instead of
    // re-parsing the whole buffer with `parse_sfc` just to read the defineArt
    // component name.
    if let Some(script_setup) = art_desc.script_setup.as_ref()
        && let Some(defined_component) =
            crate::virtual_code::find_define_art_component_name(script_setup.content)
    {
        let pascal_component = kebab_to_pascal(component_name);
        if component_name == defined_component || pascal_component == defined_component {
            return Some(component_path.to_string());
        }
    }

    let stem = std::path::Path::new(component_path)
        .file_stem()
        .and_then(|stem| stem.to_str())?;

    let pascal_component = kebab_to_pascal(component_name);
    let pascal_stem = kebab_to_pascal(stem);
    (component_name == stem || pascal_component == pascal_stem).then(|| component_path.to_string())
}

fn extract_component_metadata(
    content: &str,
    filename: &str,
    options_api: bool,
    legacy_vue2: bool,
) -> ComponentMetadata {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return ComponentMetadata {
            props: Vec::new(),
            slots: Vec::new(),
        };
    };

    let mut props = Vec::new();
    let mut slots = Vec::new();
    let mut seen_props = BTreeSet::new();
    let mut seen_slots = BTreeSet::new();

    if let Some(script_content) = descriptor
        .script_setup
        .as_ref()
        .map(|script| script.content.as_ref())
        .or_else(|| {
            descriptor
                .script
                .as_ref()
                .map(|script| script.content.as_ref())
        })
    {
        let analyzer_options = DrawerOptions {
            analyze_script: true,
            ..Default::default()
        };
        let mut analyzer = Drawer::with_options(analyzer_options);
        if legacy_vue2 {
            analyzer = analyzer.with_legacy_vue2();
        } else if options_api {
            analyzer = analyzer.with_options_api();
        }
        if descriptor.script_setup.is_some() {
            analyzer.analyze_script_setup(script_content);
        } else {
            analyzer.analyze_script_plain(script_content);
        }
        let summary = analyzer.finish();

        for prop in summary.macros.props() {
            if seen_props.insert(prop.name.to_string()) {
                props.push(ComponentProp {
                    name: prop.name.to_string(),
                    type_detail: prop.prop_type.as_ref().map(|ty| ty.to_string()),
                    required: prop.required,
                    default_value: prop.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        // defineModel<T>() introduces a prop alongside an `update:NAME`
        // event. Prop completion only knew about defineProps before, so
        // child components using defineModel showed no prop suggestions.
        // See #686.
        for model in summary.macros.models() {
            if seen_props.insert(model.name.to_string()) {
                props.push(ComponentProp {
                    name: model.name.to_string(),
                    type_detail: model.model_type.as_ref().map(|ty| ty.to_string()),
                    required: model.required,
                    default_value: model.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        if legacy_vue2 {
            for (name, binding_type) in summary.bindings.iter() {
                if binding_type == BindingType::Props && seen_props.insert(name.to_string()) {
                    props.push(ComponentProp {
                        name: name.to_string(),
                        type_detail: None,
                        required: false,
                        default_value: None,
                    });
                }
            }
        }

        for slot in summary.macros.slots() {
            let name = slot.name.to_string();
            if seen_slots.insert(name.clone()) {
                slots.push(ComponentSlot {
                    name,
                    props_type: slot.props_type.as_ref().map(|props| props.to_string()),
                });
            }
        }
    }

    if let Some(template) = descriptor.template.as_ref() {
        for slot in extract_template_slot_outlets(template.content.as_ref()) {
            if seen_slots.insert(slot.name.clone()) {
                slots.push(slot);
            }
        }
    }

    ComponentMetadata { props, slots }
}

fn prop_completion_item(prop: &ComponentProp, dynamic: bool) -> CompletionItem {
    let kebab_name = pascal_to_kebab(&prop.name);
    let label = if dynamic {
        prop.name.clone()
    } else {
        kebab_name.clone()
    };
    let insert_name = label.clone();
    let insert_text = if !dynamic && prop.type_detail.as_deref() == Some("boolean") {
        insert_name
    } else {
        format!("{insert_name}=\"$1\"")
    };
    let required = if prop.required {
        "required"
    } else {
        "optional"
    };
    let type_detail = prop.type_detail.as_deref().unwrap_or("unknown");

    let mut doc_body = format!(
        "**Prop** `{}`\n\n```typescript\n{}: {}\n```",
        prop.name, prop.name, type_detail
    );
    if let Some(ref default) = prop.default_value {
        doc_body.push_str("\n\nDefault: `");
        doc_body.push_str(default);
        doc_body.push('`');
    }

    CompletionItem {
        label,
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(format!("prop: {type_detail} ({required})")),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(format!(": {type_detail}")),
            description: Some(required.to_string()),
        }),
        insert_text: Some(insert_text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc_body,
        })),
        sort_text: Some(format!("00-prop-{kebab_name}")),
        ..Default::default()
    }
}

fn slot_completion_item(slot: &ComponentSlot, prefix: &str) -> CompletionItem {
    let after_hash = prefix.starts_with('#');
    let after_v_slot = prefix.starts_with("v-slot:");
    let label = if after_hash || after_v_slot {
        slot.name.clone()
    } else {
        format!("#{}", slot.name)
    };
    let destructure = slot
        .props_type
        .as_ref()
        .and_then(|ty| extract_slot_prop_names(ty));
    let value_snippet = match destructure {
        Some(names) if !names.is_empty() => {
            // Pre-populate the destructure with the resolved slot prop names
            // so the user gets `{ row, col }` rather than just `="$1"`.
            // The `${1:...}` placeholder lets the editor select the names.
            format!("{{ ${{1:{}}} }}", names.join(", "))
        }
        _ => "$1".to_string(),
    };
    let insert_text = if slot.props_type.is_some() {
        if after_hash || after_v_slot {
            format!("{}=\"{value_snippet}\"", slot.name)
        } else {
            format!("#{}=\"{value_snippet}\"", slot.name)
        }
    } else if after_hash || after_v_slot {
        slot.name.clone()
    } else {
        format!("#{}", slot.name)
    };

    CompletionItem {
        label,
        kind: Some(CompletionItemKind::FIELD),
        detail: Some(
            slot.props_type
                .as_ref()
                .map(|props| format!("slot props: {props}"))
                .unwrap_or_else(|| "slot".to_string()),
        ),
        insert_text: Some(insert_text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: slot.props_type.as_ref().map(|props| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Slot** `{}`\n\n```typescript\n{}\n```", slot.name, props),
            })
        }),
        sort_text: Some(format!("00-slot-{}", slot.name)),
        ..Default::default()
    }
}

const SLOT_PROPS_TYPE_PREFIX: &str = "type __VizeSlotProps = ";

/// Extract slot prop names from the already-resolved first slot parameter type
/// and return names that are safe to put into a destructuring snippet.
fn extract_slot_prop_names(ts_type: &str) -> Option<Vec<String>> {
    let source = slot_props_type_source(ts_type);
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return None;
    }

    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return None;
    };

    let mut names = Vec::new();
    collect_slot_prop_names_from_ts_type(&alias.type_annotation, &mut names);
    if names.is_empty() { None } else { Some(names) }
}

fn slot_props_type_source(ts_type: &str) -> String {
    let trimmed = ts_type.trim();
    let mut source = String::with_capacity(SLOT_PROPS_TYPE_PREFIX.len() + trimmed.len() + 1);
    source.push_str(SLOT_PROPS_TYPE_PREFIX);
    source.push_str(trimmed);
    source.push(';');
    source
}

fn collect_slot_prop_names_from_ts_type(ts_type: &TSType<'_>, names: &mut Vec<String>) {
    match ts_type {
        TSType::TSTypeLiteral(literal) => {
            for member in &literal.members {
                if let TSSignature::TSPropertySignature(property) = member
                    && let Some(name) = slot_prop_key_name(&property.key)
                {
                    names.push(name);
                }
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for ty in &intersection.types {
                collect_slot_prop_names_from_ts_type(ty, names);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            collect_slot_prop_names_from_ts_type(&parenthesized.type_annotation, names);
        }
        TSType::TSTypeReference(reference) => {
            if let Some(type_arguments) = &reference.type_arguments {
                for ty in &type_arguments.params {
                    collect_slot_prop_names_from_ts_type(ty, names);
                }
            }
        }
        _ => {}
    }
}

fn slot_prop_key_name(key: &PropertyKey<'_>) -> Option<String> {
    let name = match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str(),
        PropertyKey::StringLiteral(literal) => literal.value.as_str(),
        _ => return None,
    };
    is_valid_destructure_key(name).then(|| name.to_string())
}

fn is_valid_destructure_key(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

#[cfg(test)]
mod slot_prop_name_tests {
    use super::extract_slot_prop_names;

    #[test]
    fn extracts_slot_prop_names_with_ts_ast() {
        assert_eq!(
            extract_slot_prop_names("Readonly<{ foo: string; $bar?: number; 'not-valid': Date }>"),
            Some(vec!["foo".to_string(), "$bar".to_string()])
        );
    }

    #[test]
    fn returns_none_for_non_object_slot_props() {
        assert_eq!(extract_slot_prop_names("Props"), None);
    }
}

fn extract_template_slot_outlets(template: &str) -> Vec<ComponentSlot> {
    let mut slots = Vec::new();
    let mut seen = BTreeSet::new();
    let mut pos = 0usize;

    while let Some(relative_start) = template[pos..].find("<slot") {
        let tag_start = pos + relative_start;
        let after_name = tag_start + "<slot".len();
        if template
            .as_bytes()
            .get(after_name)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            pos = after_name;
            continue;
        }

        let Some(tag_end) = find_tag_end(template, tag_start) else {
            break;
        };
        let tag = &template[tag_start..=tag_end];
        let name = find_attr_value(tag, "name").unwrap_or_else(|| "default".to_string());
        if seen.insert(name.clone()) {
            slots.push(ComponentSlot {
                name,
                props_type: None,
            });
        }
        pos = tag_end + 1;
    }

    slots
}

#[cfg(test)]
mod cache_tests {
    use super::cached_component_metadata;
    use crate::ide::IdeContext;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn component_metadata_cache_hits_then_invalidates_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let component = dir.path().join("Widget.vue");
        std::fs::write(
            &component,
            "<script setup lang=\"ts\">\nconst props = defineProps<{ a: string }>()\n</script>\n",
        )
        .unwrap();

        let state = ServerState::new();
        let uri = Url::parse("file:///host.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template></template>".to_string(),
            1,
            "vue".to_string(),
        );
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();

        let first = cached_component_metadata(&ctx, &component).unwrap();
        let second = cached_component_metadata(&ctx, &component).unwrap();
        let prop = first
            .props
            .iter()
            .find(|prop| prop.name == "a")
            .expect("defineProps type member should be extracted");
        assert_eq!(prop.type_detail.as_deref(), Some("string"));
        assert!(prop.required);
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "an unchanged component file should hit the cache (same Arc, no re-parse)",
        );
        let first_prop_count = first.props.len();

        // Rewrite with a different length so the file stamp changes; the next
        // lookup must recompute rather than serve the stale cached parse.
        std::fs::write(
            &component,
            "<script setup lang=\"ts\">\nconst props = defineProps<{ a: string; bb: number }>()\n</script>\n",
        )
        .unwrap();

        let third = cached_component_metadata(&ctx, &component).unwrap();
        assert!(
            !std::sync::Arc::ptr_eq(&first, &third),
            "a changed component file must invalidate the cached entry",
        );
        assert!(
            third.props.len() > first_prop_count,
            "recomputed metadata should reflect the added prop ({} -> {})",
            first_prop_count,
            third.props.len(),
        );
    }
}

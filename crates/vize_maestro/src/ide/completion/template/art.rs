//! Art (`*.art.vue`) and inline `<art>` block completions: art/variant blocks,
//! their attributes and script-block snippets.

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, InsertTextFormat,
    MarkupContent, MarkupKind,
};

use crate::ide::IdeContext;
use crate::ide::completion::items;
use crate::ide::completion::{
    is_inside_art_tag, is_inside_variant_tag, should_suggest_art_block,
    should_suggest_variant_block,
};

/// Get completions for Art files (*.art.vue).
pub(crate) fn complete_art(ctx: &IdeContext) -> Option<CompletionResponse> {
    let mut items_vec = Vec::new();

    let content = &ctx.content;
    let offset = ctx.offset;
    let before_cursor = content.get(..offset.min(content.len()))?;

    if is_inside_art_tag(before_cursor) {
        items_vec.extend(art_attribute_completions());
    } else if is_inside_variant_tag(before_cursor) {
        items_vec.extend(variant_attribute_completions());
    } else if should_suggest_art_block(before_cursor) {
        items_vec.extend(art_block_completions());
    } else if should_suggest_variant_block(before_cursor) {
        items_vec.extend(variant_block_completions());
    }

    items_vec.extend(art_script_completions());

    if items_vec.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items_vec))
    }
}

/// Get completions for inline <art> blocks in regular .vue files.
pub(crate) fn complete_inline_art(ctx: &IdeContext) -> Option<CompletionResponse> {
    let mut items_vec = Vec::new();

    let content = &ctx.content;
    let offset = ctx.offset;
    let before_cursor = content.get(..offset.min(content.len()))?;

    if is_inside_art_tag(before_cursor) {
        items_vec.extend(art_attribute_completions());
    } else if is_inside_variant_tag(before_cursor) {
        items_vec.extend(variant_attribute_completions());
    } else if should_suggest_variant_block(before_cursor) {
        items_vec.extend(variant_block_completions());
    }

    if items_vec.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items_vec))
    }
}

/// Art block completions at root level.
fn art_block_completions() -> Vec<CompletionItem> {
    vec![CompletionItem {
        label: "art".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Create Art block".to_string()),
        insert_text: Some(
            "<art>\n\t<variant name=\"$1\" default>\n\t\t$0\n\t</variant>\n</art>".to_string()
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Art Block**\n\nDefines component variants. The target is inferred from the imported component used by each variant. Optional metadata uses `defineArtMeta`.\n\n```vue\n<script setup lang=\"ts\">\nimport Button from \"./Button.vue\";\ndefineArtMeta({ category: \"Components\" });\n</script>\n\n<art>\n  <variant name=\"Primary\" default>\n    <Button>Click</Button>\n  </variant>\n</art>\n```".to_string(),
        })),
        ..Default::default()
    }]
}

/// Art attribute completions inside <art> tag.
fn art_attribute_completions() -> Vec<CompletionItem> {
    vec![
        items::attr_item("title", "Optional component title override", "title=\"$1\""),
        items::attr_item("component", "Path to component file", "component=\"$1\""),
        items::attr_item("description", "Component description", "description=\"$1\""),
        items::attr_item(
            "category",
            "Component category (e.g., atoms, molecules)",
            "category=\"$1\"",
        ),
        items::attr_item("tags", "Comma-separated tags", "tags=\"$1\""),
        items::attr_item(
            "status",
            "Component status (ready, draft, deprecated)",
            "status=\"$1\"",
        ),
        items::attr_item("order", "Display order in gallery", "order=\"$1\""),
    ]
}

/// Variant block completions inside <art>.
fn variant_block_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "variant".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create variant block".to_string()),
            insert_text: Some("<variant name=\"$1\">\n\t$0\n</variant>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Variant Block**\n\nDefines a component variation with specific props.\n\n```vue\n<variant name=\"Primary\" default>\n  <Button variant=\"primary\">Click</Button>\n</variant>\n```".to_string(),
            })),
            ..Default::default()
        },
        CompletionItem {
            label: "variant with props".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create variant with typed props".to_string()),
            insert_text: Some(
                "<variant name=\"$1\" :props=\"{ $2: $3 }\">\n\t$0\n</variant>".to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Variant attribute completions inside <variant> tag.
fn variant_attribute_completions() -> Vec<CompletionItem> {
    vec![
        items::attr_item("name", "Variant name (required)", "name=\"$1\""),
        items::attr_item("default", "Mark as default variant", "default"),
        items::attr_item("props", "Typed component props", ":props=\"{ $1: $2 }\""),
        items::attr_item("attrs", "Fallthrough attributes", ":attrs=\"{ '$1': $2 }\""),
        items::attr_item(
            "play",
            "Typed interaction test",
            ":play=\"async ({ canvas, user, expect }) => { $1 }\"",
        ),
        items::attr_item(
            "viewport",
            "Viewport dimensions (WxH or WxH@scale)",
            "viewport=\"$1\"",
        ),
        items::attr_item("skip-vrt", "Skip visual regression test", "skip-vrt"),
    ]
}

/// Script block completions for Art files.
fn art_script_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "script setup".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Add script setup block".to_string()),
            insert_text: Some(
                "<script setup lang=\"ts\">\ndefineArtMeta({\n\ttitle: \"$1\",\n});\n</script>"
                    .to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "style".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Add style block".to_string()),
            insert_text: Some("<style scoped>\n$0\n</style>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

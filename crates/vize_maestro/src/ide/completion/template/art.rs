//! Art (`*.art.vue`) and inline `<art>` block completions: art/variant blocks,
//! their attributes, the `<Self>` reference, and script-block snippets.

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
    let before_cursor = &content[..offset.min(content.len())];

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
    let before_cursor = &content[..offset.min(content.len())];

    if is_inside_art_tag(before_cursor) {
        items_vec.extend(art_attribute_completions());
    } else if is_inside_variant_tag(before_cursor) {
        items_vec.extend(variant_attribute_completions());
    } else if should_suggest_variant_block(before_cursor) {
        items_vec.extend(variant_block_completions());
        items_vec.push(self_component_completion());
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
            value: "**Art Block**\n\nDefines component variants. Metadata and target component are declared with `defineArt` in `<script setup>`.\n\n```vue\n<script setup lang=\"ts\">\ndefineArt(\"./Button.vue\", { title: \"Button\" });\n</script>\n\n<art>\n  <variant name=\"Primary\" default>\n    <Button>Click</Button>\n  </variant>\n</art>\n```".to_string(),
        })),
        ..Default::default()
    }]
}

/// Art attribute completions inside <art> tag.
fn art_attribute_completions() -> Vec<CompletionItem> {
    vec![
        items::attr_item("title", "Component title (required)", "title=\"$1\""),
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
            label: "variant with args".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create variant with args".to_string()),
            insert_text: Some(
                "<variant name=\"$1\" args='{\"$2\": $3}'>\n\t$0\n</variant>".to_string(),
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
        items::attr_item("args", "Props as JSON", "args='{\"$1\": $2}'"),
        items::attr_item(
            "viewport",
            "Viewport dimensions (WxH or WxH@scale)",
            "viewport=\"$1\"",
        ),
        items::attr_item("skip-vrt", "Skip visual regression test", "skip-vrt"),
    ]
}

/// Completion item for <Self> component reference in inline art blocks.
fn self_component_completion() -> CompletionItem {
    CompletionItem {
        label: "Self".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("Reference to the host component".to_string()),
        insert_text: Some("<Self $1>$0</Self>".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**`<Self>`**\n\nReferences the host component in inline art blocks.\nReplaced with the component name at build time.".to_string(),
        })),
        ..Default::default()
    }
}

/// Script block completions for Art files.
fn art_script_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "script setup".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Add script setup block".to_string()),
            insert_text: Some(
                "<script setup lang=\"ts\">\ndefineArt(\"$1\", {\n\ttitle: \"$2\",\n});\n</script>"
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

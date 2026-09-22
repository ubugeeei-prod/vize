//! CSS module class completions (`$style.|`).

use std::collections::BTreeSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};

use crate::ide::IdeContext;

/// `<style module>` populates a template-scope `$style` object whose
/// properties are the declared class names. When the cursor sits at
/// `$style.|` we surface those names instead of the usual directive list.
pub(crate) fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let before = &ctx.content[..ctx.offset.min(ctx.content.len())];
    let trimmed = before.trim_end_matches([' ', '\t']);
    if !trimmed.ends_with("$style.") {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let mut classes = BTreeSet::new();
    for style in descriptor.styles.iter() {
        let attrs = &style.attrs;
        if !attrs.contains_key("module") {
            continue;
        }
        for class in extract_css_class_names(&style.content) {
            classes.insert(class);
        }
    }
    if classes.is_empty() {
        return None;
    }
    Some(
        classes
            .into_iter()
            .map(|name| {
                #[allow(clippy::disallowed_macros)]
                CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("CSS module class".to_string()),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("`{name}` from `<style module>`."),
                    })),
                    sort_text: Some(format!("0{name}")),
                    ..Default::default()
                }
            })
            .collect(),
    )
}

/// Extract class selector names from raw CSS content. Approximate but good
/// enough for completion — matches `.identifier` outside attribute selectors.
fn extract_css_class_names(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_class_ident_byte(bytes[end]) {
                end += 1;
            }
            if end > start {
                out.push(css[start..end].to_string());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn is_class_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

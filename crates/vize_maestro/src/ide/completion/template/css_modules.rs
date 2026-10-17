//! CSS module class completions (`$style.|`).

use std::collections::BTreeSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_canon::css_classes::authored_local_css_classes;

use crate::ide::IdeContext;

/// `<style module>` populates a template-scope `$style` object whose
/// properties are the declared class names. When the cursor sits at
/// `$style.|` we surface those names instead of the usual directive list.
pub(crate) fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let before = ctx.content.get(..ctx.offset.min(ctx.content.len()))?;
    let trimmed = before.trim_end_matches([' ', '\t']);
    if !trimmed.ends_with("$style.") {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let mut classes = BTreeSet::new();
    for style in descriptor.styles.iter() {
        if style.module.as_deref() != Some("$style")
            || style.src.is_some()
            || style
                .lang
                .as_deref()
                .is_some_and(|language| !language.eq_ignore_ascii_case("css"))
        {
            continue;
        }
        if let Some(names) = authored_local_css_classes(&style.content) {
            classes.extend(names);
        }
    }
    Some(
        classes
            .into_iter()
            .map(|name| {
                #[expect(clippy::disallowed_macros, reason = "tower-lsp payload fields take std `String`, which `cstr!` does not produce")]
                CompletionItem {
                    label: name.to_string(),
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

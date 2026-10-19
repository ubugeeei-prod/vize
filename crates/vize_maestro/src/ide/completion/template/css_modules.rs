//! CSS module class completions (`$style.|`).

use std::collections::BTreeSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_canon::css_classes::{authored_local_css_classes, authored_local_scss_classes};

use crate::ide::IdeContext;

/// `<style module>` populates a template-scope `$style` object (or the
/// name given by `module="name"`) whose properties are the declared class
/// names. When the cursor sits at `$style.|` or `$style.par|` we surface
/// those names instead of the usual directive list.
pub(crate) fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let before = ctx.content.get(..ctx.offset.min(ctx.content.len()))?;
    let receiver = before
        .trim_end_matches(is_identifier_char)
        .strip_suffix('.')?;
    let module = receiver
        .rsplit(|character| !is_identifier_char(character))
        .next()
        .filter(|module| !module.is_empty())?;
    let descriptor = ctx.descriptor()?;
    if !descriptor
        .styles
        .iter()
        .any(|style| style.module.as_deref() == Some(module))
    {
        return None;
    }
    let mut classes = BTreeSet::new();
    for style in descriptor.styles.iter() {
        if style.module.as_deref() != Some(module) || style.src.is_some() {
            continue;
        }
        let names = match style.lang.as_deref() {
            None => authored_local_css_classes(&style.content),
            Some(language) if language.eq_ignore_ascii_case("css") => {
                authored_local_css_classes(&style.content)
            }
            Some(language) if language.eq_ignore_ascii_case("scss") => {
                authored_local_scss_classes(&style.content)
            }
            Some(_) => None,
        };
        classes.extend(names.into_iter().flatten());
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

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '$')
}

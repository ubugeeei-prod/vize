//! CSS module class completions (`$style.|`).

use std::collections::BTreeSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_l1::css::AuthoredClasses;

use crate::ide::IdeContext;

/// `<style module>` populates a template-scope `$style` object (or the
/// name given by `module="name"`) whose properties are the declared class
/// names. When the cursor sits at `$style.|` or `$style.par|` we surface
/// those names instead of the usual directive list.
pub(crate) fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let module = module_receiver(ctx.content.get(..ctx.offset.min(ctx.content.len()))?)?;
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
        let names = vize_l1::container::lang::style_lang(style.lang.as_deref())
            .ok()
            .and_then(|language| language.authored_classes(&style.content))
            .map(AuthoredClasses::into_names);
        classes.extend(names.into_iter().flatten());
    }
    Some(
        classes
            .into_iter()
            .map(|name| class_completion(&name))
            .collect(),
    )
}

fn module_receiver(before: &str) -> Option<&str> {
    let receiver = before
        .trim_end_matches(is_identifier_char)
        .strip_suffix('.')?;
    receiver
        .rsplit(|character| !is_identifier_char(character))
        .next()
        .filter(|module| !module.is_empty())
}

fn class_completion(name: &str) -> CompletionItem {
    #[expect(
        clippy::disallowed_macros,
        reason = "tower-lsp payload fields take std `String`, which `cstr!` does not produce"
    )]
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
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '$')
}

#[cfg(feature = "experimental-style-module-products")]
mod products;
#[cfg(feature = "experimental-style-module-products")]
pub use products::{StyleModuleRefusal, StyleModuleToolProducts};

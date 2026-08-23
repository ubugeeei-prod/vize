//! `v-slot` scope prop lookup in authored templates.
#![allow(clippy::disallowed_types)]

use vize_croquis::{Analyzer, AnalyzerOptions, ScopeKind};

use super::{TemplateScopeBinding, TemplateScopeBindingKind};
use crate::ide::IdeContext;

pub(super) fn binding_at(ctx: &IdeContext<'_>, word: &str) -> Option<TemplateScopeBinding> {
    if word.is_empty() {
        return None;
    }

    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let template = descriptor.template.as_ref()?;
    if ctx.offset < template.loc.start || ctx.offset > template.loc.end {
        return None;
    }

    let cursor = ctx.offset - template.loc.start;
    let cursor = u32::try_from(cursor).ok()?;
    let allocator = vize_carton::Allocator::new();
    let (ast, _) = vize_armature::parse(&allocator, template.content.as_ref());
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if ctx.state.legacy_vue2_enabled() {
        analyzer = analyzer.with_legacy_vue2();
    }
    analyzer.analyze_template(&ast);
    let summary = analyzer.finish();

    let (name, binding, kind) = summary
        .scopes
        .bindings_visible_at(cursor)
        .into_iter()
        .find(|(name, _, _)| *name == word)?;
    if kind != ScopeKind::VSlot {
        return None;
    }

    let declaration = usize::try_from(binding.declaration_offset).ok()?;
    Some(TemplateScopeBinding {
        name: name.to_string(),
        start: template.loc.start + declaration,
        end: template.loc.start + declaration + name.len(),
        kind: TemplateScopeBindingKind::SlotProp,
    })
}

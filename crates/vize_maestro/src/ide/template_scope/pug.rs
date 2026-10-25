//! Authored scope queries through the shared derived-Pug compatibility map.

use vize_atelier_sfc::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor, analyze_sfc_descriptor_with_context_legacy_vue2,
    analyze_sfc_descriptor_with_context_options_api,
};
use vize_croquis::{Croquis, ScopeData, ScopeKind};
use vize_l1_to_l2::lower::pug::{PugRendering, PugTemplate, derive_template_source_with};

use super::{TemplateScopeBinding, TemplateScopeBindingKind};
use crate::ide::IdeContext;

pub(in crate::ide) struct PugScope {
    template: PugTemplate,
    croquis: Croquis,
    start: usize,
}

pub(in crate::ide) fn is_pug(ctx: &IdeContext<'_>) -> bool {
    ctx.block_type == Some(crate::virtual_code::BlockType::Template)
        && ctx
            .descriptor()
            .and_then(|descriptor| descriptor.template.as_ref())
            .is_some_and(|template| {
                template.src.is_none()
                    && template.lang.as_deref().is_some_and(|lang| {
                        lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade")
                    })
            })
}

fn scope<'a>(ctx: &'a IdeContext<'_>) -> Option<&'a PugScope> {
    ctx.pug_scope
        .get_or_init(|| {
            if !is_pug(ctx) {
                return None;
            }
            let descriptor = ctx.descriptor()?;
            let authored = descriptor.template.as_ref()?;
            let template = vize_l0::profile!(
                "maestro.template_scope.pug.derive",
                derive_template_source_with(&authored.content, PugRendering::AuthoredOrder)
            );
            if template.has_errors() {
                return None;
            }
            let allocator = vize_l0::Allocator::new();
            let (root, errors) = vize_armature::parse(&allocator, &template.html);
            if !errors.is_empty() {
                return None;
            }
            let mut options = SfcCroquisOptions::full();
            options.template_is_derived = true;
            let croquis = vize_l0::profile!("maestro.template_scope.pug.analyze", {
                if ctx.state.legacy_vue2_enabled() {
                    analyze_sfc_descriptor_with_context_legacy_vue2(
                        descriptor,
                        Some(&root),
                        options,
                    )
                    .croquis
                } else if ctx.state.options_api_enabled() {
                    analyze_sfc_descriptor_with_context_options_api(
                        descriptor,
                        Some(&root),
                        options,
                    )
                    .croquis
                } else {
                    analyze_sfc_descriptor(descriptor, Some(&root), options)
                }
            });
            Some(Box::new(PugScope {
                template,
                croquis,
                start: authored.loc.start,
            }))
        })
        .as_deref()
}

pub(in crate::ide) fn analyzed<'a>(ctx: &'a IdeContext<'_>) -> Option<(&'a Croquis, u32)> {
    analyzed_at(ctx, false)
}

pub(in crate::ide) fn analyzed_for_completion<'a>(
    ctx: &'a IdeContext<'_>,
) -> Option<(&'a Croquis, u32)> {
    analyzed_at(ctx, true)
}

fn analyzed_at<'a>(ctx: &'a IdeContext<'_>, completion: bool) -> Option<(&'a Croquis, u32)> {
    let scope = scope(ctx)?;
    let local = u32::try_from(ctx.offset.checked_sub(scope.start)?).ok()?;
    let local = if completion {
        scope.template.map.verbatim_cursor_to_html(local)?
    } else {
        scope.template.map.verbatim_offset_to_html(local)?
    };
    Some((&scope.croquis, local))
}

pub(super) fn binding_at(ctx: &IdeContext<'_>, word: &str) -> Option<TemplateScopeBinding> {
    let scope = scope(ctx)?;
    let (_, cursor) = analyzed(ctx)?;
    let (name, binding, kind) = super::bindings_visible_at(&scope.croquis, cursor)
        .into_iter()
        .find(|(name, _, kind)| {
            *name == word && matches!(kind, ScopeKind::VFor | ScopeKind::VSlot)
        })?;
    let kind = match kind {
        ScopeKind::VSlot => TemplateScopeBindingKind::SlotProp,
        ScopeKind::VFor => {
            let declaration = scope.croquis.scopes.iter().find(|scope| {
                scope.kind == ScopeKind::VFor
                    && scope.bindings().any(|(name, candidate)| {
                        name == word && candidate.declaration_offset == binding.declaration_offset
                    })
            })?;
            let ScopeData::VFor(data) = declaration.data() else {
                return None;
            };
            if data.key_alias.as_deref() == Some(word) {
                TemplateScopeBindingKind::Key
            } else if data.index_alias.as_deref() == Some(word) {
                TemplateScopeBindingKind::Index
            } else {
                TemplateScopeBindingKind::Value
            }
        }
        _ => return None,
    };
    let declaration = scope.template.map.to_pug(vize_l0::Span::new(
        binding.declaration_offset,
        binding
            .declaration_offset
            .checked_add(u32::try_from(name.len()).ok()?)?,
    ));
    let start = scope.start.checked_add(declaration.start as usize)?;
    let end = scope.start.checked_add(declaration.end as usize)?;
    // A synthesized/decoded range must not become a different authored name.
    if ctx.content.get(start..end)? != name {
        return None;
    }
    Some(TemplateScopeBinding {
        name: name.into(),
        start,
        end,
        kind,
    })
}

#[cfg(test)]
mod tests;

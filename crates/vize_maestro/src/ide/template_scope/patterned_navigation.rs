//! Keep checker-only navigation local to symbols affected by pattern scopes.

use vize_croquis::{
    Croquis, ScopeData, ScopeKind,
    drawer::{extract_identifier_refs_oxc, extract_identifiers_oxc},
};

use crate::ide::IdeContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Navigation {
    Ordinary,
    Shared,
    Local,
}

pub(crate) fn needs_patterned_navigation(ctx: &IdeContext<'_>) -> bool {
    classify(ctx) != Navigation::Ordinary
}

/// Structural edits must not merge disabled pattern bindings into outer names.
pub(crate) fn needs_structural_pattern_navigation(ctx: &IdeContext<'_>) -> bool {
    if !ctx.content.contains("v-match") && !ctx.content.contains("v-when") {
        return false;
    }
    classify_with_patterns(ctx, true) != Navigation::Ordinary
}

fn classify(ctx: &IdeContext<'_>) -> Navigation {
    classify_with_patterns(ctx, ctx.state.patterned_template_enabled())
}

fn classify_with_patterns(ctx: &IdeContext<'_>, patterned: bool) -> Navigation {
    if !patterned || !ctx.uri.path().ends_with(".vue") {
        return Navigation::Ordinary;
    }
    let Some((croquis, start)) = super::analysis::analyze_with_patterns(ctx, patterned) else {
        return Navigation::Shared;
    };
    if croquis
        .pattern_diagnostics
        .iter()
        .any(|diagnostic| !diagnostic.warning)
    {
        return Navigation::Shared;
    }
    let matches: Vec<_> = croquis
        .scopes
        .iter()
        .filter(|scope| scope.kind == ScopeKind::VMatch)
        .collect();
    if matches.is_empty() {
        return Navigation::Ordinary;
    }
    let offset = ctx.offset.checked_sub(start).map(|offset| offset as u32);
    let inside =
        offset.is_some_and(|offset| matches.iter().any(|scope| scope.span.contains(offset)));
    let Some(word) = crate::ide::token_at_offset(&ctx.content, ctx.offset, |byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
    }) else {
        return if inside {
            Navigation::Shared
        } else {
            Navigation::Ordinary
        };
    };

    if inside && let Some(offset) = offset {
        let declaration = croquis.scopes.iter().any(|scope| {
            matches!(
                scope.data(),
                ScopeData::VWhen(_) | ScopeData::VFor(_) | ScopeData::VSlot(_)
            ) && scope.bindings().any(|(name, binding)| {
                name == word
                    && offset >= binding.declaration_offset
                    && offset <= binding.declaration_offset + name.len() as u32
            })
        });
        let visible = super::bindings_visible_at(&croquis, offset)
            .iter()
            .any(|(name, _, kind)| {
                *name == word
                    && matches!(
                        kind,
                        ScopeKind::VWhen
                            | ScopeKind::VFor
                            | ScopeKind::VSlot
                            | ScopeKind::Callback
                            | ScopeKind::EventHandler
                    )
            });
        if declaration || visible && is_identifier_reference(ctx, &croquis, start, offset, &word) {
            return Navigation::Local;
        }
    }

    // Outer symbols used by a pattern, and outer names shadowed by an arm,
    // also require semantic identity. Keep their workspace routing intact.
    let affected = inside
        || croquis
            .scopes
            .iter()
            .any(|scope| scope.kind == ScopeKind::VWhen && scope.get_binding(&word).is_some())
        || croquis.template_expressions.iter().any(|expression| {
            matches
                .iter()
                .any(|scope| scope.span.contains(expression.start))
                && extract_identifiers_oxc(&expression.content)
                    .iter()
                    .any(|name| *name == word)
        });
    if affected {
        Navigation::Shared
    } else {
        Navigation::Ordinary
    }
}

fn is_identifier_reference(
    ctx: &IdeContext<'_>,
    croquis: &Croquis,
    start: usize,
    offset: u32,
    word: &str,
) -> bool {
    let contains = |content: &str, at: u32, end: u32| {
        if offset < at || offset > end {
            return false;
        }
        let raw = ctx.content.get(start + at as usize..start + end as usize);
        extract_identifier_refs_oxc(content)
            .iter()
            .any(|reference| {
                if reference.name != word {
                    return false;
                }
                let relative = raw
                    .filter(|raw| raw.len() != content.len())
                    .map_or(reference.offset, |raw| {
                        vize_armature::patterns::attribute_source_offset(raw, reference.offset)
                    });
                offset >= at + relative && offset <= at + relative + word.len() as u32
            })
    };
    croquis
        .template_expressions
        .iter()
        .any(|expression| contains(&expression.content, expression.start, expression.end))
        || croquis.scopes.iter().any(|scope| {
            let ScopeData::VFor(data) = scope.data() else {
                return false;
            };
            croquis
                .scopes
                .v_for_source_offset(scope.id)
                .is_some_and(|at| contains(&data.source, at, at + data.source.len() as u32))
        })
}

#[cfg(test)]
mod tests;

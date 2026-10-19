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
mod tests {
    use super::*;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    const SOURCE: &str = r#"<script setup lang="ts">
const ordinary = 1
const rows = 'outer'
const result = { rows: [1, 2] }
</script>
<template>
  <section v-match="result">
    <template v-when="{ const rows } if (rows.length > 0)">
      <p v-for="row in rows">{{ row.toFixed() }}{{ rows.length }}</p>
    </template>
    <p v-when="_"/>
  </section>
  <p>{{ ordinary }}</p>
</template>"#;

    fn enabled_state() -> (tempfile::TempDir, ServerState) {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(
            project.path().join("vite.config.mjs"),
            r#"export default { vue: {"experimentals":{"patternedTemplate":true}} };"#,
        )
        .unwrap();
        let state = ServerState::new();
        state.load_workspace_config(project.path());
        (project, state)
    }

    #[test]
    fn classification_separates_local_shared_and_unaffected_symbols() {
        let (project, state) = enabled_state();
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        for (needle, expected) in [
            ("ordinary =", Navigation::Ordinary),
            ("ordinary }}", Navigation::Ordinary),
            ("rows =", Navigation::Shared),
            ("result =", Navigation::Shared),
            ("result\">", Navigation::Shared),
            ("rows } if", Navigation::Local),
            ("rows.length >", Navigation::Local),
            ("rows.length }}", Navigation::Local),
            ("rows\">", Navigation::Local),
            ("row.toFixed", Navigation::Local),
            ("row in", Navigation::Local),
        ] {
            let ctx =
                IdeContext::testing(&state, &uri, SOURCE.find(needle).unwrap(), SOURCE.into());
            assert_eq!(classify(&ctx), expected, "{needle}");
        }
        let unicode = SOURCE.replace("rows", "\u{6570}\u{5024}");
        for (needle, expected) in [
            ("\u{6570}\u{5024} =", Navigation::Shared),
            ("\u{6570}\u{5024}.length }}", Navigation::Local),
        ] {
            let ctx =
                IdeContext::testing(&state, &uri, unicode.find(needle).unwrap(), unicode.clone());
            assert_eq!(classify(&ctx), expected);
        }
        let malformed = SOURCE.replace("{ const rows }", "{ const rows, const rows }");
        let property = SOURCE.replace("{{ rows.length }}", "{{ rows.row }}");
        let ctx = IdeContext::testing(&state, &uri, property.find("row }}").unwrap(), property);
        assert_eq!(classify(&ctx), Navigation::Shared);
        let ctx = IdeContext::testing(
            &state,
            &uri,
            malformed.find("rows.length }}").unwrap(),
            malformed,
        );
        assert_eq!(classify(&ctx), Navigation::Shared);
    }

    #[test]
    fn opt_out_and_pattern_like_text_do_not_change_ordinary_routing() {
        let (project, enabled) = enabled_state();
        let disabled = ServerState::new();
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        let ctx = IdeContext::testing(
            &disabled,
            &uri,
            SOURCE.find("rows.length }}").unwrap(),
            SOURCE.into(),
        );
        assert_eq!(classify(&ctx), Navigation::Ordinary);
        let source = "<script setup>const value = '<template v-match>'; </script><template><!-- v-match=\"value\" -->{{ value }}</template>";
        let ctx = IdeContext::testing(
            &enabled,
            &uri,
            source.find("value }}").unwrap(),
            source.into(),
        );
        assert_eq!(classify(&ctx), Navigation::Ordinary);
        let uri = Url::from_file_path(project.path().join("plain.ts")).unwrap();
        let ctx = IdeContext::testing(&enabled, &uri, 6, "const value = 1".into());
        assert_eq!(classify(&ctx), Navigation::Ordinary);
    }

    #[cfg(feature = "native")]
    #[test]
    fn experimental_opt_in_keeps_ordinary_reference_and_rename_fallbacks() {
        use crate::ide::{ReferencesService, RenameService};
        crate::runtime::block_on(async {
            let (project, enabled) = enabled_state();
            let disabled = ServerState::new();
            let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
            for source in [
                SOURCE,
                "<script setup>const ordinary = 1</script><template>{{ ordinary }}</template>",
            ] {
                let offset = source.find("ordinary }}").unwrap();
                for state in [&disabled, &enabled] {
                    state
                        .documents
                        .open(uri.clone(), source.into(), 1, "vue".into());
                    state.update_virtual_docs(&uri, source);
                }
                let baseline = IdeContext::new(&disabled, &uri, offset).unwrap();
                let ctx = IdeContext::new(&enabled, &uri, offset).unwrap();
                let references = ReferencesService::references(&baseline, true).unwrap();
                assert_eq!(references.len(), 2);
                let rename = RenameService::rename(&baseline, "changed").unwrap();
                let prepare = RenameService::prepare_rename(&baseline).unwrap();
                assert_eq!(
                    ReferencesService::references(&ctx, true),
                    Some(references.clone())
                );
                assert_eq!(RenameService::rename(&ctx, "changed"), Some(rename.clone()));
                for bridge in [
                    None,
                    Some(std::sync::Arc::new(vize_canon::CorsaBridge::new())),
                ] {
                    assert_eq!(
                        ReferencesService::references_with_corsa(&ctx, true, bridge.clone()).await,
                        Some(references.clone())
                    );
                    assert_eq!(
                        RenameService::prepare_rename_with_corsa(&ctx, bridge.clone()).await,
                        Some(prepare.clone())
                    );
                    assert_eq!(
                        RenameService::rename_with_corsa(&ctx, "changed", bridge).await,
                        Some(rename.clone())
                    );
                }
            }
        });
    }
}

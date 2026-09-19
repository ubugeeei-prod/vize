//! Descriptor-aware scope analysis shared by authored editor operations.

use vize_atelier_sfc::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor, analyze_sfc_descriptor_with_context_legacy_vue2,
    analyze_sfc_descriptor_with_context_options_api,
};
use vize_croquis::{Croquis, ScopeBinding, ScopeData, ScopeKind};

use crate::ide::IdeContext;

pub(crate) fn analyze(ctx: &IdeContext<'_>) -> Option<(Croquis, usize)> {
    analyze_with_patterns(ctx, ctx.state.patterned_template_enabled())
}

pub(super) fn analyze_with_patterns(
    ctx: &IdeContext<'_>,
    patterned: bool,
) -> Option<(Croquis, usize)> {
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let descriptor = vize_atelier_sfc::prepare_root_patterned_template(
        &descriptor,
        patterned,
        false,
        Default::default(),
    )
    .ok()?;
    let allocator = vize_s0::Allocator::new();
    let template = descriptor.template.as_ref();
    let parsed = template.map(|template| vize_armature::parse(&allocator, &template.content));
    let ast = parsed.as_ref().map(|(root, _)| root);
    let mut options = SfcCroquisOptions::full();
    options.analyzer_options.experimental_patterned_template = patterned;
    let croquis = if ctx.state.legacy_vue2_enabled() {
        analyze_sfc_descriptor_with_context_legacy_vue2(&descriptor, ast, options).croquis
    } else if ctx.state.options_api_enabled() {
        analyze_sfc_descriptor_with_context_options_api(&descriptor, ast, options).croquis
    } else {
        analyze_sfc_descriptor(&descriptor, ast, options)
    };
    Some((croquis, template.map_or(0, |template| template.loc.start)))
}

pub(crate) fn bindings_visible_at(
    croquis: &Croquis,
    offset: u32,
) -> Vec<(&str, ScopeBinding, ScopeKind)> {
    let scope = croquis
        .scopes
        .iter()
        .filter(|scope| {
            matches!(
                scope.kind,
                ScopeKind::VFor
                    | ScopeKind::VSlot
                    | ScopeKind::EventHandler
                    | ScopeKind::Callback
                    | ScopeKind::VMatch
                    | ScopeKind::VWhen
            ) && scope.span.contains(offset)
                && match scope.data() {
                    ScopeData::VFor(data) => !croquis
                        .scopes
                        .v_for_source_offset(scope.id)
                        .is_some_and(|start| {
                            offset >= start && offset <= start + data.source.len() as u32
                        }),
                    ScopeData::VWhen(data) => {
                        offset < data.offset || offset > data.offset + data.arm.pattern.span.end
                    }
                    _ => true,
                }
        })
        .min_by_key(|scope| (scope.span.len(), std::cmp::Reverse(scope.id.as_u32())));
    scope.map_or_else(Vec::new, |scope| {
        croquis.scopes.bindings_visible_from(scope.id)
    })
}

#[cfg(feature = "native")]
pub(crate) fn may_reference_style_binding(ctx: &IdeContext<'_>, word: &str) -> bool {
    if !ctx.state.patterned_template_enabled()
        || ctx.block_type != Some(crate::virtual_code::BlockType::Template)
    {
        return true;
    }
    let Some((croquis, start)) = analyze(ctx) else {
        return false;
    };
    let offset = ctx.offset.saturating_sub(start) as u32;
    // A pattern declaration is not visible in its own value expressions, but
    // querying the declaration itself must still retain its local identity.
    let declaration = croquis.scopes.iter().any(|scope| {
        matches!(scope.data(), ScopeData::VWhen(_))
            && scope.bindings().any(|(name, binding)| {
                name == word
                    && offset >= binding.declaration_offset
                    && offset <= binding.declaration_offset + name.len() as u32
            })
    });
    !declaration
        && !bindings_visible_at(&croquis, offset)
            .iter()
            .any(|(name, _, kind)| {
                *name == word
                    && matches!(
                        kind,
                        ScopeKind::VFor
                            | ScopeKind::VSlot
                            | ScopeKind::VWhen
                            | ScopeKind::Callback
                            | ScopeKind::EventHandler
                    )
            })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pattern_guard_keeps_its_arm_bindings_despite_script_offset_overlap() {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(
            project.path().join("vize.config.json"),
            r#"{"experimentals":{"patternedTemplate":true}}"#,
        )
        .unwrap();
        let state = crate::server::ServerState::new();
        state.load_workspace_config(project.path());
        let uri =
            tower_lsp::lsp_types::Url::from_file_path(project.path().join("App.vue")).unwrap();
        let source = r#"<script setup lang="ts">
const rows = 'outer'
const result = {} as { kind: 'ok'; rows: number[]; marker: number } | { kind: 'err'; message: string }
</script>
<template v-match="result">
  <template v-when="{ kind: 'ok', const rows, marker: const armOnly } if (rows.length > 0)">
    <p>{{ rows.length }}{{ armOnly }}</p>
  </template>
  <p v-when="_"/>
</template>"#;
        let ctx = IdeContext::with_content(
            &state,
            &uri,
            source.find("rows.length >").unwrap(),
            source.into(),
        );
        let (croquis, start) = analyze(&ctx).unwrap();
        assert!(
            croquis.pattern_diagnostics.is_empty(),
            "{:?}",
            croquis.pattern_diagnostics
        );
        let visible = bindings_visible_at(&croquis, (ctx.offset - start) as u32);
        assert!(
            visible.iter().any(
                |(name, _, kind)| *name == "armOnly" && *kind == vize_croquis::ScopeKind::VWhen
            ),
            "{visible:?}"
        );
    }
}

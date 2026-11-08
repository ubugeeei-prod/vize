//! Template uses of shadowed setup bindings, as uses of the authored binding.
//!
//! The template closure shadows a ref-like setup binding with an unwrapped
//! `var`, so a template use names that generated declaration. A host without
//! Vize's semantic links sees two unrelated symbols. One type query per use,
//! in setup scope before the shadow, names the authored binding at the same
//! template position; a type query reads no value, so it adds no diagnostic.
//! A separate const declaration in template scope owns hover, preserving both
//! the authored declaration kind and the template's unwrapped type.

use vize_carton::{String, append};
use vize_croquis::drawer::{AuthoredSourceOffsets, extract_identifier_uses_checked};
use vize_croquis::{Croquis, ScopeData, ScopeId, ScopeKind, TemplateExpressionKind};

use crate::virtual_ts::generated_names::{TEMPLATE_HOVER_PREFIX, TEMPLATE_USE_PREFIX};
use crate::virtual_ts::{VizeMapping, VizeSemanticLink, VizeSemanticLinkKind};

/// Emit one type query per template use of a `shadowed` binding.
pub(super) fn emit_navigation(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    shadowed: &[&str],
    template_source: &str,
    template_offset: u32,
) {
    let uses = collect(summary, shadowed, template_source);
    if uses.is_empty() {
        return;
    }
    ts.push_str("  void (0 as unknown as [\n");
    for (name, start) in uses {
        append!(*ts, "    {TEMPLATE_USE_PREFIX}");
        let generated = ts.len();
        append!(*ts, "{name},\n");
        let authored = (template_offset + start) as usize;
        mappings.push(VizeMapping::new(
            generated..generated + name.len(),
            authored..authored + name.len(),
        ));
    }
    ts.push_str("  ]);\n");
}

/// Emit an authored Options API setup-return anchor beside its navigation uses.
///
/// The setup-return local lives inside `setup()`, so a setup-scope `typeof name`
/// cannot see it. A small block avoids colliding with same-spelled module
/// bindings while retaining the exact identifier spelling required by the
/// content-mapper's whole-symbol projection.
#[expect(clippy::too_many_arguments, reason = "independent projection inputs")]
pub(super) fn emit_options_api_navigation(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    summary: &Croquis,
    shadowed: &[String],
    template_source: &str,
    template_offset: u32,
    script: Option<&str>,
    script_source_offset: &dyn Fn(usize) -> usize,
) {
    for name in shadowed {
        let uses = collect(summary, &[name.as_str()], template_source);
        if uses.is_empty() {
            continue;
        }
        ts.push_str("  {\n    const ");
        let declaration = ts.len();
        append!(*ts, "{name}: __R_{name} = (void 0) as any;\n");

        if let Some((start, end)) = crate::virtual_ts::script_facts::binding_span(summary, name)
            && script.and_then(|script| script.get(start as usize..end as usize))
                == Some(name.as_str())
        {
            let original = script_source_offset(start as usize)..script_source_offset(end as usize);
            let generated = declaration..declaration + name.len();
            if let Some(source_range) =
                super::super::script_module::mapped_binding_range(mappings, &original)
            {
                semantic_links.push(VizeSemanticLink {
                    source_range,
                    target_range: generated.clone(),
                    kind: VizeSemanticLinkKind::VueOptionsApiBinding,
                });
            }
            mappings.push(VizeMapping::new(generated, original));
        }

        ts.push_str("    void (0 as unknown as [\n");
        for (use_name, start) in uses {
            append!(*ts, "      {TEMPLATE_USE_PREFIX}");
            let generated = ts.len();
            append!(*ts, "{use_name},\n");
            let authored = (template_offset + start) as usize;
            mappings.push(VizeMapping::new(
                generated..generated + use_name.len(),
                authored..authored + use_name.len(),
            ));
        }
        ts.push_str("    ]);\n  }\n");
    }
}

/// Emit a const declaration with the template-visible type for every use.
pub(super) fn emit_hover(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    shadowed: &[&str],
    template_source: &str,
    template_offset: u32,
    type_annotation: impl Fn(&str) -> Option<String>,
) {
    for (name, start) in collect(summary, shadowed, template_source) {
        let Some(annotation) = type_annotation(name.as_str()) else {
            continue;
        };
        append!(*ts, "    {{\n      {TEMPLATE_HOVER_PREFIX}");
        let generated = ts.len();
        append!(
            *ts,
            "{name}: {annotation} = (void 0) as any;\n      void {name};\n    }}\n"
        );
        let authored = (template_offset + start) as usize;
        mappings.push(VizeMapping::new(
            generated..generated + name.len(),
            authored..authored + name.len(),
        ));
    }
}

fn collect(summary: &Croquis, shadowed: &[&str], template_source: &str) -> Vec<(String, u32)> {
    let mut uses = Vec::new();
    let mut collect =
        |source: &str, authored: &str, start: u32, scope: Option<ScopeId>, text: bool| {
            if !shadowed.iter().any(|name| source.contains(name)) {
                return;
            }
            let authored_offsets = (authored != source).then(|| {
                if text {
                    AuthoredSourceOffsets::text(authored)
                } else {
                    AuthoredSourceOffsets::attribute(authored)
                }
            });
            for reference in extract_identifier_uses_checked(source)
                .into_iter()
                .flatten()
            {
                let name = reference.name.as_str();
                if shadowed.contains(&name) && !bound_in_template(summary, scope, name) {
                    let offset = authored_offsets
                        .as_ref()
                        .map_or(reference.offset, |offsets| offsets.offset(reference.offset));
                    uses.push((reference.name, start + offset));
                }
            }
        };
    for expression in &summary.template_expressions {
        let authored = template_source
            .get(expression.start as usize..expression.end as usize)
            .unwrap_or(expression.content.as_str());
        collect(
            expression.content.as_str(),
            authored,
            expression.start,
            Some(expression.scope_id),
            expression.kind == TemplateExpressionKind::Interpolation,
        );
    }
    // A `v-for` source reads the scope around its loop.
    for scope in summary.scopes.iter() {
        if let ScopeData::VFor(data) = scope.data()
            && let Some(span) = summary.scopes.v_for_source_span(scope.id)
        {
            let start = span.start;
            let authored = template_source
                .get(span.start as usize..span.end as usize)
                .unwrap_or(data.source.as_str());
            collect(data.source.as_str(), authored, start, scope.parent(), false);
        }
    }
    uses.sort_unstable_by_key(|(_, start)| *start);
    uses
}

/// Whether a template scope from `scope` outward declares `name`.
fn bound_in_template(summary: &Croquis, mut scope: Option<ScopeId>, name: &str) -> bool {
    while let Some(id) = scope {
        let Some(current) = summary.scopes.get_scope(id) else {
            return false;
        };
        if current.has_binding(name) {
            return matches!(
                current.kind,
                ScopeKind::VFor
                    | ScopeKind::VSlot
                    | ScopeKind::EventHandler
                    | ScopeKind::Callback
                    | ScopeKind::Closure
                    | ScopeKind::Block
            );
        }
        scope = current.parent();
    }
    false
}

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
use vize_croquis::drawer::extract_identifier_uses_checked;
use vize_croquis::{Croquis, ScopeData, ScopeId, ScopeKind};

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::generated_names::{TEMPLATE_HOVER_PREFIX, TEMPLATE_USE_PREFIX};

/// Emit one type query per template use of a `shadowed` binding.
pub(super) fn emit_navigation(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    shadowed: &[&str],
    template_offset: u32,
) {
    let uses = collect(summary, shadowed);
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

/// Emit a const declaration with the template-visible type for every use.
pub(super) fn emit_hover(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    shadowed: &[&str],
    template_offset: u32,
    type_annotation: impl Fn(&str) -> Option<String>,
) {
    for (name, start) in collect(summary, shadowed) {
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

fn collect(summary: &Croquis, shadowed: &[&str]) -> Vec<(String, u32)> {
    let mut uses = Vec::new();
    let mut collect = |source: &str, start: u32, scope: Option<ScopeId>| {
        if !shadowed.iter().any(|name| source.contains(name)) {
            return;
        }
        for reference in extract_identifier_uses_checked(source)
            .into_iter()
            .flatten()
        {
            let name = reference.name.as_str();
            if shadowed.contains(&name) && !bound_in_template(summary, scope, name) {
                uses.push((reference.name, start + reference.offset));
            }
        }
    };
    for expression in &summary.template_expressions {
        collect(
            expression.content.as_str(),
            expression.start,
            Some(expression.scope_id),
        );
    }
    // A `v-for` source reads the scope around its loop.
    for scope in summary.scopes.iter() {
        if let ScopeData::VFor(data) = scope.data()
            && let Some(start) = summary.scopes.v_for_source_offset(scope.id)
        {
            collect(data.source.as_str(), start, scope.parent());
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

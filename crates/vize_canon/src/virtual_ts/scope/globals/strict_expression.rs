//! Strict template-context refs discovered from template expression AST scans.

use vize_carton::{FxHashSet, String};
use vize_croquis::{Croquis, analyzer::extract_identifier_refs_oxc};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::{
    emit_header, emit_strict_template_context_ref, is_declared_template_context_name,
    is_template_instance_global_name,
    member_root::{is_call_root_occurrence, is_member_root_occurrence},
    strict_candidate::is_strict_template_context_candidate,
    template_scope::{is_inside_template_scope, is_visible_template_binding},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_strict_expression_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    options: &VirtualTsOptions,
    template_prop_names: &FxHashSet<String>,
    type_export_names: &FxHashSet<&str>,
    seen_names: &mut FxHashSet<String>,
    seen_strict_occurrences: &mut FxHashSet<(String, usize)>,
    emitted_header: &mut bool,
) {
    for expr in &summary.template_expressions {
        for ident in extract_identifier_refs_oxc(expr.content.as_str()) {
            let name = ident.name.as_str();
            let local_start = expr.start + ident.offset;
            let head = expr.content.as_str()[..ident.offset as usize].trim_end();
            let member_root = is_member_root_occurrence(summary, local_start, name);
            let call_root = is_call_root_occurrence(summary, local_start, name);
            let scoped_bare_ref =
                !member_root && !call_root && is_inside_template_scope(summary, local_start);
            if is_template_instance_global_name(name)
                || !is_strict_template_context_candidate(name)
                || (!member_root && !call_root && !scoped_bare_ref)
                || is_declared_template_context_name(name, options)
                || template_prop_names.contains(name)
                || type_export_names.contains(name)
                || is_visible_template_binding(summary, name, local_start)
                || head.ends_with('.')
                || head.ends_with("?.")
            {
                continue;
            }
            let src_start = (template_offset + local_start) as usize;
            if !seen_strict_occurrences.insert((String::from(name), src_start)) {
                continue;
            }
            let already_declared = member_root || !seen_names.insert(String::from(name));
            emit_header(ts, emitted_header);
            emit_strict_template_context_ref(
                ts,
                mappings,
                name,
                src_start,
                src_start + name.len(),
                already_declared,
            );
        }
    }
}

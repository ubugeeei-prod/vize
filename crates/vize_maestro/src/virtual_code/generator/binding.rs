//! Semantic helpers for determining which `<script setup>` bindings are
//! referenced by template expressions.

use std::collections::BTreeSet;
use vize_croquis::analyzer::extract_identifiers_oxc;

use super::super::script_code::extract_simple_bindings_demand;
use super::TemplateExpression;

pub(super) fn template_used_script_bindings(
    script_content: &str,
    expressions: &[TemplateExpression],
) -> Vec<String> {
    template_used_script_bindings_demand(script_content, expressions, false).0
}

pub(super) fn template_used_script_bindings_demand(
    script_content: &str,
    expressions: &[TemplateExpression],
    capture: bool,
) -> (
    Vec<String>,
    Option<vize_croquis::binding_occurrences::BindingOccurrences>,
) {
    let (bindings, packet) = extract_simple_bindings_demand(script_content, true, capture);
    let script_bindings = bindings.into_iter().collect::<BTreeSet<_>>();
    if script_bindings.is_empty() {
        return (Vec::new(), packet);
    }

    let mut used = BTreeSet::new();
    for expression in expressions {
        for identifier in extract_identifiers_oxc(&expression.text) {
            if script_bindings.contains(identifier.as_str()) {
                used.insert(identifier.to_string());
            }
        }
    }

    (used.into_iter().collect(), packet)
}

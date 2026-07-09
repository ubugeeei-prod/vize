//! Semantic helpers for determining which `<script setup>` bindings are
//! referenced by template expressions.

use std::collections::BTreeSet;
use vize_croquis::analyzer::extract_identifiers_oxc;

use super::TemplateExpression;
use super::extract_simple_bindings;

pub(super) fn template_used_script_bindings(
    script_content: &str,
    expressions: &[TemplateExpression],
) -> Vec<String> {
    let script_bindings = extract_simple_bindings(script_content, true)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if script_bindings.is_empty() {
        return Vec::new();
    }

    let mut used = BTreeSet::new();
    for expression in expressions {
        for identifier in extract_identifiers_oxc(&expression.text) {
            if script_bindings.contains(identifier.as_str()) {
                used.insert(identifier.to_string());
            }
        }
    }

    used.into_iter().collect()
}

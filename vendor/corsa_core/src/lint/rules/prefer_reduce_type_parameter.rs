use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::{
    lint::helpers::{callee_property_name, child_list, member_object},
    utils::is_array_like_type_texts,
};

/// Type-aware rule that prefers Array#reduce type parameters over default-value assertions.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferReduceTypeParameterRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "preferTypeParameter",
        description: "Unnecessary assertion: Array#reduce accepts a type parameter for the default value.",
    },
    RuleMessage {
        id: "moveTypeParameter",
        description: "Move the assertion type to the Array#reduce type parameter.",
    },
];
const LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for PreferReduceTypeParameterRule {
    fn name(&self) -> &'static str {
        "prefer-reduce-type-parameter"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer Array#reduce type parameters over default-value assertions."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "CallExpression"
            || callee_property_name(Some(node)).as_deref() != Some("reduce")
        {
            return;
        }
        let Some(initializer_assertion) = child_list(node, "arguments").get(1) else {
            return;
        };
        if !matches!(
            initializer_assertion.kind.as_str(),
            "TSAsExpression" | "TSTypeAssertion"
        ) {
            return;
        }
        let Some(callee) = node.child("callee") else {
            return;
        };
        let Some(object) = member_object(callee) else {
            return;
        };
        if object.kind != "ArrayExpression" && !is_array_like_type_texts(&object.type_texts) {
            return;
        }
        let Some(type_annotation) = type_annotation_text(initializer_assertion) else {
            return;
        };
        if is_bare_type_parameter(type_annotation) {
            return;
        }
        let Some(initializer) = initializer_assertion.child("expression") else {
            return;
        };
        if !is_obviously_assignable_initializer(initializer, type_annotation) {
            return;
        }

        ctx.report_with_suggestions(
            "preferTypeParameter",
            initializer_assertion.range,
            move_type_parameter_suggestion(node, callee, initializer_assertion, initializer)
                .into_iter()
                .collect(),
        );
    }
}

fn type_annotation_text(node: &LintNode) -> Option<&str> {
    node.field_str("__typeAnnotationText")
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn is_bare_type_parameter(type_annotation: &str) -> bool {
    let mut chars = type_annotation.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_obviously_assignable_initializer(initializer: &LintNode, type_annotation: &str) -> bool {
    if initializer
        .type_texts
        .iter()
        .any(|text| same_type_text(text, type_annotation))
    {
        return true;
    }
    if initializer.kind == "ArrayExpression" {
        return is_array_like_type_texts(&[type_annotation]);
    }
    if initializer.kind == "ObjectExpression"
        && child_list(initializer, "properties").is_empty()
        && is_object_like_type_annotation(type_annotation)
    {
        return true;
    }
    false
}

fn same_type_text(left: &str, right: &str) -> bool {
    normalize_type_text(left) == normalize_type_text(right)
}

fn normalize_type_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn is_object_like_type_annotation(type_annotation: &str) -> bool {
    let text = type_annotation.trim();
    text.starts_with("Record<")
        || text.starts_with('{')
        || text == "object"
        || text == "{}"
        || text == "any"
        || text == "unknown"
}

fn move_type_parameter_suggestion(
    call: &LintNode,
    callee: &LintNode,
    assertion: &LintNode,
    initializer: &LintNode,
) -> Option<LintSuggestion> {
    let type_annotation = type_annotation_text(assertion)?;
    let remove_range = match assertion.kind.as_str() {
        "TSAsExpression" => TextRange::new(initializer.range.end, assertion.range.end),
        "TSTypeAssertion" => TextRange::new(assertion.range.start, initializer.range.start),
        _ => return None,
    };
    if !remove_range.is_valid() {
        return None;
    }

    let mut fixes = vec![LintFix::remove_range(remove_range)];
    if !has_type_parameters(call) {
        fixes.push(LintFix::replace_range(
            TextRange::new(callee.range.end, callee.range.end),
            format!("<{type_annotation}>"),
        ));
    }
    Some(LintSuggestion {
        message_id: "moveTypeParameter".to_owned(),
        message: "Move the assertion type to the Array#reduce type parameter.".to_owned(),
        fixes,
    })
}

fn has_type_parameters(node: &LintNode) -> bool {
    node.child("typeArguments").is_some()
        || node.child("typeParameters").is_some()
        || node.child("typeParameterInstantiation").is_some()
}

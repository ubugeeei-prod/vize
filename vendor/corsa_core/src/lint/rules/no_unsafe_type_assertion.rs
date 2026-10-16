use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::utils::{has_unsafe_any_flow, split_top_level_type_text};

/// Type-aware rule that rejects unsafe type assertions.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeTypeAssertionRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unsafeOfAnyTypeAssertion",
        description: "Unsafe assertion from any detected: consider using type guards or a safer assertion.",
    },
    RuleMessage {
        id: "unsafeToAnyTypeAssertion",
        description: "Unsafe assertion to any detected: consider using a more specific type to ensure safety.",
    },
    RuleMessage {
        id: "unsafeToUnconstrainedTypeAssertion",
        description: "Unsafe type assertion to an unconstrained type parameter.",
    },
    RuleMessage {
        id: "unsafeTypeAssertion",
        description: "Unsafe type assertion: the asserted type is more narrow than the original type.",
    },
    RuleMessage {
        id: "unsafeTypeAssertionAssignableToConstraint",
        description: "Unsafe type assertion to a type parameter that may be instantiated with a narrower subtype.",
    },
];
const LISTENERS: &[&str] = &["TSAsExpression", "TSTypeAssertion"];

impl RustLintRule for NoUnsafeTypeAssertionRule {
    fn name(&self) -> &'static str {
        "no-unsafe-type-assertion"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow unsafe type assertions."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if !matches!(node.kind.as_str(), "TSAsExpression" | "TSTypeAssertion") {
            return;
        }
        let Some(expression) = node.child("expression") else {
            return;
        };
        let Some(asserted_type) = asserted_type_text(node) else {
            return;
        };
        let source_type_texts = source_type_texts(expression);
        if source_type_texts
            .iter()
            .any(|source| same_type_text(source, asserted_type))
        {
            return;
        }

        let target_texts = [asserted_type];
        let source_contains_any = has_unsafe_any_flow(&source_type_texts, &[] as &[&str]);
        let target_contains_any = has_unsafe_any_flow(&target_texts, &[] as &[&str]);

        if target_contains_any && !source_contains_any {
            ctx.report("unsafeToAnyTypeAssertion", node.range);
            return;
        }
        if source_contains_any && !is_permissive_asserted_type(asserted_type) {
            ctx.report("unsafeOfAnyTypeAssertion", node.range);
            return;
        }
        if is_bare_type_parameter(asserted_type)
            && !source_type_texts
                .iter()
                .any(|source| same_type_text(source, asserted_type))
        {
            ctx.report("unsafeToUnconstrainedTypeAssertion", node.range);
            return;
        }
        if source_type_texts
            .iter()
            .any(|source| is_likely_narrowing_assertion(source, asserted_type))
        {
            ctx.report("unsafeTypeAssertion", node.range);
        }
    }
}

fn source_type_texts(node: &LintNode) -> Vec<String> {
    if matches!(node.kind.as_str(), "TSAsExpression" | "TSTypeAssertion")
        && let Some(text) = asserted_type_text(node)
    {
        return vec![text.to_owned()];
    }
    node.type_texts.clone()
}

fn asserted_type_text(node: &LintNode) -> Option<&str> {
    node.field_str("__typeAnnotationText")
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn is_permissive_asserted_type(text: &str) -> bool {
    matches!(normalize_type_text(text).as_str(), "any" | "unknown")
}

fn is_bare_type_parameter(type_annotation: &str) -> bool {
    let mut chars = type_annotation.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && !matches!(type_annotation, "Array" | "Promise" | "ReadonlyArray")
}

fn is_likely_narrowing_assertion(source: &str, target: &str) -> bool {
    let source = source.trim();
    let target = target.trim();
    if source.is_empty()
        || target.is_empty()
        || same_type_text(source, target)
        || is_permissive_asserted_type(target)
    {
        return false;
    }
    if normalize_type_text(target) == "never" {
        return normalize_type_text(source) != "never";
    }
    if normalize_type_text(source) == "unknown" {
        return true;
    }
    if is_readonly_array_to_mutable_array(source, target) {
        return true;
    }
    if is_union_narrowing(source, target) {
        return true;
    }
    if let Some((source_base, source_args)) = generic_parts(source) {
        if let Some((target_base, target_args)) = generic_parts(target) {
            if same_container_family(source_base, target_base)
                && source_args.len() == target_args.len()
                && source_args
                    .iter()
                    .zip(target_args.iter())
                    .any(|(source_arg, target_arg)| {
                        is_likely_narrowing_assertion(source_arg, target_arg)
                    })
            {
                return true;
            }
        }
    }
    if let (Some(source_item), Some(target_item)) =
        (array_item_type(source), array_item_type(target))
    {
        return is_likely_narrowing_assertion(source_item, target_item);
    }
    source == "Function" && (target.contains("=>") || target.starts_with("()"))
}

fn is_union_narrowing(source: &str, target: &str) -> bool {
    let source_parts = normalized_union_parts(source);
    let target_parts = normalized_union_parts(target);
    if source_parts.len() <= 1 && target_parts.len() <= 1 {
        return false;
    }
    if target_parts
        .iter()
        .any(|target| target == "unknown" || target == "any")
    {
        return false;
    }
    let source_has_target = target_parts
        .iter()
        .all(|target| source_parts.iter().any(|source| source == target));
    let target_missing_source = source_parts
        .iter()
        .any(|source| !target_parts.iter().any(|target| target == source));
    source_has_target && target_missing_source
}

fn normalized_union_parts(text: &str) -> Vec<String> {
    split_top_level_type_text(text, '|')
        .into_iter()
        .map(|part| normalize_type_text(&part))
        .collect()
}

fn array_item_type(text: &str) -> Option<&str> {
    let text = text.trim();
    text.strip_prefix("readonly ")
        .unwrap_or(text)
        .strip_suffix("[]")
}

fn is_readonly_array_to_mutable_array(source: &str, target: &str) -> bool {
    source.trim_start().starts_with("readonly ")
        && array_item_type(source).is_some()
        && target.trim_start().strip_prefix("readonly ").is_none()
        && array_item_type(target).is_some()
}

fn generic_parts(text: &str) -> Option<(&str, Vec<String>)> {
    let trimmed = text.trim();
    let start = trimmed.find('<')?;
    if !trimmed.ends_with('>') {
        return None;
    }
    let base = trimmed[..start].trim();
    let inner = &trimmed[start + 1..trimmed.len() - 1];
    Some((base, split_top_level_type_text(inner, ',')))
}

fn same_container_family(left: &str, right: &str) -> bool {
    left == right
        || (matches!(left, "Array" | "ReadonlyArray") && matches!(right, "Array" | "ReadonlyArray"))
        || (matches!(left, "Promise" | "PromiseLike") && matches!(right, "Promise" | "PromiseLike"))
}

fn same_type_text(left: &str, right: &str) -> bool {
    normalize_type_text(left) == normalize_type_text(right)
}

fn normalize_type_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

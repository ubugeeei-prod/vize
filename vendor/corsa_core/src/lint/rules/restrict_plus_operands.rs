use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::rule_option_bool;
use crate::utils::{TypeTextKind, classify_type_text};

/// Type-aware rule that requires plus operands to be explicitly compatible.
#[derive(Clone, Copy, Debug, Default)]
pub struct RestrictPlusOperandsRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "invalid",
        description: "Operands of + must be compatible primitive values.",
    },
    RuleMessage {
        id: "mismatched",
        description: "Operands of + operations must be of the same type.",
    },
];
const LISTENERS: &[&str] = &["AssignmentExpression", "BinaryExpression"];

impl RustLintRule for RestrictPlusOperandsRule {
    fn name(&self) -> &'static str {
        "restrict-plus-operands"
    }

    fn docs_description(&self) -> &'static str {
        "Require plus operands to be explicitly compatible."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let options = RestrictPlusOperandsOptions::from_node(node);
        match node.kind.as_str() {
            "AssignmentExpression" => {
                if options.skip_compound_assignments || node.field_str("operator") != Some("+=") {
                    return;
                }
                let Some(left) = node.child("left") else {
                    return;
                };
                let Some(right) = node.child("right") else {
                    return;
                };
                report_if_invalid(ctx, node, left, right, options);
            }
            "BinaryExpression" => {
                if node.field_str("operator") != Some("+") {
                    return;
                }
                let Some(left) = node.child("left") else {
                    return;
                };
                let Some(right) = node.child("right") else {
                    return;
                };
                report_if_invalid(ctx, node, left, right, options);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RestrictPlusOperandsOptions {
    allow_any: bool,
    allow_boolean: bool,
    allow_nullish: bool,
    allow_number_and_string: bool,
    allow_reg_exp: bool,
    skip_compound_assignments: bool,
}

impl RestrictPlusOperandsOptions {
    fn from_node(node: &LintNode) -> Self {
        Self {
            allow_any: rule_option_bool(node, "allowAny").unwrap_or(true),
            allow_boolean: rule_option_bool(node, "allowBoolean").unwrap_or(true),
            allow_nullish: rule_option_bool(node, "allowNullish").unwrap_or(true),
            allow_number_and_string: rule_option_bool(node, "allowNumberAndString").unwrap_or(true),
            allow_reg_exp: rule_option_bool(node, "allowRegExp").unwrap_or(false),
            skip_compound_assignments: rule_option_bool(node, "skipCompoundAssignments")
                .unwrap_or(false),
        }
    }
}

fn report_if_invalid(
    ctx: &mut RuleContext<'_>,
    node: &LintNode,
    left: &LintNode,
    right: &LintNode,
    options: RestrictPlusOperandsOptions,
) {
    let left_kind = classify_node_type(left);
    let right_kind = classify_node_type(right);
    if is_allowed(left_kind, right_kind, options) {
        return;
    }
    let message_id = if matches!(
        (left_kind, right_kind),
        (TypeTextKind::Other, _)
            | (_, TypeTextKind::Other)
            | (TypeTextKind::Unknown, _)
            | (_, TypeTextKind::Unknown)
    ) {
        "invalid"
    } else {
        "mismatched"
    };
    ctx.report(message_id, node.range);
}

fn classify_node_type(node: &LintNode) -> TypeTextKind {
    classify_type_text(node.type_texts.first().map(String::as_str))
}

fn is_allowed(
    left_kind: TypeTextKind,
    right_kind: TypeTextKind,
    options: RestrictPlusOperandsOptions,
) -> bool {
    if left_kind == right_kind
        && matches!(
            left_kind,
            TypeTextKind::Bigint | TypeTextKind::Number | TypeTextKind::String
        )
    {
        return true;
    }
    if options.allow_number_and_string
        && matches!(
            (left_kind, right_kind),
            (TypeTextKind::String, TypeTextKind::Number)
                | (TypeTextKind::Number, TypeTextKind::String)
        )
    {
        return true;
    }
    if left_kind == TypeTextKind::String && is_string_companion(right_kind, options) {
        return true;
    }
    if right_kind == TypeTextKind::String && is_string_companion(left_kind, options) {
        return true;
    }
    false
}

fn is_string_companion(kind: TypeTextKind, options: RestrictPlusOperandsOptions) -> bool {
    matches!(
        kind,
        TypeTextKind::Any if options.allow_any
    ) || matches!(
        kind,
        TypeTextKind::Boolean if options.allow_boolean
    ) || matches!(
        kind,
        TypeTextKind::Nullish if options.allow_nullish
    ) || matches!(
        kind,
        TypeTextKind::Regexp if options.allow_reg_exp
    )
}

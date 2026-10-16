use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::utils::split_type_text;

/// Type-aware rule that flags conditions that are always truthy, always falsy,
/// `never`, always nullish, or comparisons between literal values.
///
/// This is a faithful port of the upstream tsgolint `no-unnecessary-condition`
/// rule. The upstream rule leans heavily on the TypeScript checker (raw type
/// flags, indexed-access resolution, optional-chain origin analysis). The parts
/// that map cleanly onto rendered type texts are ported here with the same
/// message IDs and the same truthy/falsy/never/nullish decision logic.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryConditionRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "alwaysTruthy",
        description: "Unnecessary conditional, value is always truthy.",
    },
    RuleMessage {
        id: "alwaysFalsy",
        description: "Unnecessary conditional, value is always falsy.",
    },
    RuleMessage {
        id: "never",
        description: "Unnecessary conditional, value is `never`.",
    },
    RuleMessage {
        id: "alwaysTruthyFunc",
        description: "This callback should return a conditional, but return is always truthy.",
    },
    RuleMessage {
        id: "alwaysFalsyFunc",
        description: "This callback should return a conditional, but return is always falsy.",
    },
    RuleMessage {
        id: "neverNullish",
        description: "Unnecessary optional chain on a non-nullish value.",
    },
    RuleMessage {
        id: "neverOptionalChain",
        description: "Unnecessary optional chain on a non-nullish value.",
    },
    RuleMessage {
        id: "noStrictNullCheck",
        description: "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    },
    RuleMessage {
        id: "comparisonBetweenLiteralTypes",
        description: "Unnecessary comparison between literal values.",
    },
    RuleMessage {
        id: "noOverlapBooleanExpression",
        description: "This condition will always return the same value since the types have no overlap.",
    },
    RuleMessage {
        id: "alwaysNullish",
        description: "Unnecessary conditional, value is always nullish.",
    },
    RuleMessage {
        id: "typeGuardAlreadyIsType",
        description: "Type predicate is unnecessary as the parameter type already satisfies the predicate.",
    },
];

const LISTENERS: &[&str] = &[
    "IfStatement",
    "WhileStatement",
    "DoWhileStatement",
    "ForStatement",
    "ConditionalExpression",
    "LogicalExpression",
    "AssignmentExpression",
    "BinaryExpression",
    "SwitchStatement",
];

impl RustLintRule for NoUnnecessaryConditionRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-condition"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow conditions that are always truthy, always falsy, never, or always nullish."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "IfStatement" => {
                check_node(ctx, node.child("test"), false);
            }
            "WhileStatement" | "DoWhileStatement" => {
                check_loop(ctx, node, node.child("test"));
            }
            "ForStatement" => {
                check_loop(ctx, node, node.child("test"));
            }
            "ConditionalExpression" => {
                check_node(ctx, node.child("test"), false);
            }
            "LogicalExpression" => match node.field_str("operator") {
                Some("&&") | Some("||") => {
                    check_node(ctx, node.child("left"), false);
                }
                Some("??") => {
                    check_node_for_nullish(ctx, node.child("left"));
                }
                _ => {}
            },
            "AssignmentExpression" => match node.field_str("operator") {
                Some("&&=") | Some("||=") => {
                    check_node(ctx, node.child("left"), false);
                }
                Some("??=") => {
                    check_node_for_nullish(ctx, node.child("left"));
                }
                _ => {}
            },
            "BinaryExpression" => {
                if matches!(
                    node.field_str("operator"),
                    Some("<" | ">" | "<=" | ">=" | "==" | "===" | "!=" | "!==")
                ) {
                    check_comparison(ctx, node, node.child("left"), node.child("right"));
                }
            }
            "SwitchStatement" => {
                // Each `case` clause is equivalent to `discriminant === test`.
                let Some(discriminant) = node.child("discriminant") else {
                    return;
                };
                for case in node.child_list("cases").unwrap_or(&[]) {
                    if let Some(test) = case.child("test") {
                        check_comparison(ctx, test, Some(discriminant), Some(test));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Mirrors the upstream loop-condition handling, honouring
/// `allowConstantLoopConditions` (default `"never"`).
fn check_loop(ctx: &mut RuleContext<'_>, owner: &LintNode, test: Option<&LintNode>) {
    let Some(test) = test else { return };
    let mode = allow_constant_loop_conditions(owner);
    if mode == "only-allowed-literals" && is_allowed_constant_literal(test) {
        return;
    }
    if mode == "always" {
        if is_true_keyword(test) {
            return;
        }
        if is_true_literal_type(&test.type_texts) {
            return;
        }
    }
    check_node(ctx, Some(test), false);
}

/// Faithful port of upstream `checkNode`: unwraps `!`, descends into the right
/// side of `&&`/`||`, then evaluates the truthiness of the expression's type.
fn check_node(
    ctx: &mut RuleContext<'_>,
    expression: Option<&LintNode>,
    is_unary_not_argument: bool,
) {
    let Some(expression) = expression else { return };

    // `!expr` flips the meaning; descend into the operand.
    if expression.kind == "UnaryExpression" && expression.field_str("operator") == Some("!") {
        check_node(ctx, expression.child("argument"), !is_unary_not_argument);
        return;
    }

    // `a && b` / `a || b` only evaluate the right operand's truthiness.
    if expression.kind == "LogicalExpression" {
        match expression.field_str("operator") {
            Some("&&") | Some("||") => {
                check_node(ctx, expression.child("right"), false);
                return;
            }
            // `??` is handled elsewhere, not as a conditional.
            _ => {}
        }
    }

    let type_texts = &expression.type_texts;
    if type_texts.is_empty() || is_conditional_always_necessary(type_texts) {
        return;
    }

    // The host may render the same type more than once (the narrowed literal
    // plus its widened base). Each text describes the SAME type, so the first
    // rendering that yields a definite verdict wins; merging them as if they
    // were union parts would dissolve narrowed literals back into their base.
    let (is_truthy, is_falsy) = type_texts
        .iter()
        .map(|text| check_type_condition(std::slice::from_ref(text)))
        .find(|(truthy, falsy)| *truthy || *falsy)
        .unwrap_or((false, false));

    if is_never_type(type_texts) {
        ctx.report("never", expression.range);
        return;
    }

    if is_falsy {
        if is_unary_not_argument {
            ctx.report("alwaysTruthy", expression.range);
        } else {
            ctx.report("alwaysFalsy", expression.range);
        }
        return;
    }
    if is_truthy {
        if is_unary_not_argument {
            ctx.report("alwaysFalsy", expression.range);
        } else {
            ctx.report("alwaysTruthy", expression.range);
        }
    }
}

/// Faithful port of upstream `checkNodeForNullish` (the `??` / `??=` path).
fn check_node_for_nullish(ctx: &mut RuleContext<'_>, node: Option<&LintNode>) {
    let Some(node) = node else { return };
    let type_texts = &node.type_texts;
    if type_texts.is_empty() || is_conditional_always_necessary(type_texts) {
        return;
    }

    if is_never_type(type_texts) {
        ctx.report("never", node.range);
        return;
    }
    if is_always_nullish_type(type_texts) {
        ctx.report("alwaysNullish", node.range);
        return;
    }

    if !is_nullish_type(type_texts) {
        ctx.report("neverNullish", node.range);
    }
}

/// Faithful port of upstream `checkIfBoolExpressionIsNecessaryConditional` for
/// the literal-vs-literal comparison case. The no-overlap branch needs the raw
/// checker type flags and is intentionally not approximated here.
fn check_comparison(
    ctx: &mut RuleContext<'_>,
    report: &LintNode,
    left: Option<&LintNode>,
    right: Option<&LintNode>,
) {
    let (Some(left), Some(right)) = (left, right) else {
        return;
    };
    if left.type_texts.is_empty() || right.type_texts.is_empty() {
        return;
    }
    if is_static_value(&left.type_texts) && is_static_value(&right.type_texts) {
        ctx.report("comparisonBetweenLiteralTypes", report.range);
    }
}

// ---------------------------------------------------------------------------
// Type-text predicates (ports of the upstream checker.Type_flags helpers).
// ---------------------------------------------------------------------------

/// Splits each rendered type text into its top-level union/intersection parts.
fn type_parts(type_texts: &[String]) -> Vec<String> {
    let mut parts = Vec::new();
    for text in type_texts {
        for part in split_type_text(text) {
            let trimmed = part.trim().trim_start_matches("readonly ").trim();
            if !trimmed.is_empty() {
                parts.push(trimmed.to_owned());
            }
        }
    }
    parts
}

/// Port of upstream `isConditionalAlwaysNecessary`: `any`, `unknown`, or a
/// type variable means the truthiness cannot be determined statically.
fn is_conditional_always_necessary(type_texts: &[String]) -> bool {
    type_parts(type_texts)
        .iter()
        .any(|part| is_indeterminate_atom(part))
}

/// Port of upstream `isIndeterminateType`: `any`, `unknown`, type parameters,
/// and `keyof T` index types cannot be statically resolved. We treat a single
/// capitalised identifier (with no recognised primitive meaning) as a possible
/// type parameter, mirroring the conservative upstream behaviour.
fn is_indeterminate_atom(part: &str) -> bool {
    let part = part.trim();
    if part == "any" || part == "unknown" {
        return true;
    }
    if part.starts_with("keyof ") {
        return true;
    }
    // A bare single-letter uppercase identifier (T, K, U ...) is most likely a
    // type parameter whose constraint we cannot see from the rendered text.
    let is_bare_ident = !part.is_empty()
        && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && part.chars().next().is_some_and(|c| c.is_ascii_uppercase());
    is_bare_ident && part.len() <= 2 && !is_known_object_like_atom(part)
}

fn is_never_type(type_texts: &[String]) -> bool {
    // `never` collapses a union to nothing; if any rendered text is exactly
    // `never`, or every part is `never`, treat it as `never`.
    let parts = type_parts(type_texts);
    if parts.is_empty() {
        return false;
    }
    parts.iter().all(|part| part == "never")
}

/// Port of upstream `isAlwaysNullishType`: the type can only be nullish.
fn is_always_nullish_type(type_texts: &[String]) -> bool {
    let parts = type_parts(type_texts);
    if parts.is_empty() {
        return false;
    }
    parts
        .iter()
        .all(|part| matches!(part.as_str(), "null" | "undefined" | "void"))
}

/// Port of upstream `isNullishType`: any union member can be nullish.
fn is_nullish_type(type_texts: &[String]) -> bool {
    type_parts(type_texts)
        .iter()
        .any(|part| matches!(part.as_str(), "null" | "undefined" | "void"))
}

/// Port of upstream `toStaticValue`: the type has a single static value
/// (boolean/string/number/bigint literal, or `null`/`undefined`).
fn is_static_value(type_texts: &[String]) -> bool {
    let parts = type_parts(type_texts);
    parts.len() == 1 && is_static_atom(&parts[0])
}

fn is_static_atom(part: &str) -> bool {
    matches!(part, "true" | "false" | "null" | "undefined")
        || is_string_literal_atom(part)
        || is_number_literal_atom(part)
        || is_bigint_literal_atom(part)
}

/// Port of upstream `checkTypeCondition`: returns `(is_truthy, is_falsy)`.
fn check_type_condition(type_texts: &[String]) -> (bool, bool) {
    let parts = type_parts(type_texts);
    if parts.is_empty() {
        return (false, false);
    }

    // Union semantics: all parts must agree for the whole to be truthy/falsy.
    // Intersection of literal-narrowing brands (e.g. `string & {}`) is handled
    // by treating the textual `&` split the same way the upstream walks union
    // parts: each atom is evaluated and the overall result is the conjunction.
    let mut all_truthy = true;
    let mut all_falsy = true;
    for part in &parts {
        let (part_truthy, part_falsy) = atom_condition(part);
        if !part_truthy {
            all_truthy = false;
        }
        if !part_falsy {
            all_falsy = false;
        }
    }
    (all_truthy, all_falsy)
}

/// Truthiness of a single atomic type text.
fn atom_condition(part: &str) -> (bool, bool) {
    let part = part.trim();
    match part {
        "never" => return (false, true),
        "null" | "undefined" | "void" => return (false, true),
        "true" => return (true, false),
        "false" => return (false, true),
        "object" => return (true, false),
        "0" | "0n" | "-0" | "NaN" => return (false, true),
        "\"\"" | "''" | "``" => return (false, true),
        // Generic primitive types are indeterminate.
        "string" | "number" | "bigint" | "boolean" | "symbol" | "any" | "unknown" => {
            return (false, false);
        }
        _ => {}
    }

    if is_string_literal_atom(part) {
        // Non-empty string literal (empty handled above).
        return (true, false);
    }
    if is_number_literal_atom(part) {
        return number_atom_condition(part);
    }
    if is_bigint_literal_atom(part) {
        let digits = &part[..part.len() - 1];
        if digits == "0" {
            return (false, true);
        }
        return (true, false);
    }

    // Object/array/function/branded types render as structural text and are
    // always truthy in JavaScript.
    if is_known_object_like_atom(part) {
        return (true, false);
    }

    (false, false)
}

fn number_atom_condition(part: &str) -> (bool, bool) {
    match part.parse::<f64>() {
        Ok(value) if value == 0.0 || value.is_nan() => (false, true),
        Ok(_) => (true, false),
        Err(_) => (false, false),
    }
}

fn is_known_object_like_atom(part: &str) -> bool {
    let part = part.trim();
    part.starts_with('{')
        || part.starts_with('[')
        || part.starts_with('(')
        || part.ends_with("[]")
        || part.starts_with("Array<")
        || part.starts_with("ReadonlyArray<")
        || part.contains("=>")
        // A branded/object intersection like `string & {}` keeps the object
        // atom; structural names (PascalCase with a recognised shape) are
        // treated as object-like only when they clearly are.
        || (part.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && part.len() > 2
            && part != "Function")
}

fn is_string_literal_atom(part: &str) -> bool {
    let bytes = part.as_bytes();
    part.len() >= 2
        && ((bytes[0] == b'"' && part.ends_with('"'))
            || (bytes[0] == b'\'' && part.ends_with('\''))
            || (bytes[0] == b'`' && part.ends_with('`')))
}

fn is_number_literal_atom(part: &str) -> bool {
    !part.is_empty() && part.parse::<f64>().is_ok()
}

fn is_bigint_literal_atom(part: &str) -> bool {
    part.ends_with('n') && part.len() > 1 && part[..part.len() - 1].parse::<i128>().is_ok()
}

fn is_true_literal_type(type_texts: &[String]) -> bool {
    let parts = type_parts(type_texts);
    parts.len() == 1 && parts[0] == "true"
}

// ---------------------------------------------------------------------------
// Loop-condition option handling.
// ---------------------------------------------------------------------------

/// Reads `allowConstantLoopConditions` from `__ruleOptions`. Accepts the legacy
/// boolean form (`true` -> "always") and the string form. Defaults to "never".
fn allow_constant_loop_conditions(node: &LintNode) -> String {
    let Some(options) = node
        .fields
        .get("__ruleOptions")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
    else {
        return "never".to_owned();
    };
    match options.get("allowConstantLoopConditions") {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(serde_json::Value::Bool(true)) => "always".to_owned(),
        _ => "never".to_owned(),
    }
}

fn is_true_keyword(node: &LintNode) -> bool {
    node.kind == "Literal" && node.field_bool("value") == Some(true)
}

/// Port of upstream `isAllowedConstantLiteral`: `true`, `false`, `0`, `1`.
fn is_allowed_constant_literal(node: &LintNode) -> bool {
    if node.kind != "Literal" {
        return false;
    }
    if let Some(value) = node.field_bool("value") {
        let _ = value;
        return true;
    }
    matches!(node.field_f64("value"), Some(v) if v == 0.0 || v == 1.0)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::NoUnnecessaryConditionRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        // Construct the rule directly rather than through the registry, since
        // the staged file is not yet wired into `with_default_type_aware_rules`.
        let rule = NoUnnecessaryConditionRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn if_node(test_kind: &str, test_type: &str) -> LintNode {
        serde_json::from_value(json!({
            "kind": "IfStatement",
            "range": { "start": 0, "end": 20 },
            "children": {
                "test": {
                    "kind": test_kind,
                    "range": { "start": 4, "end": 10 },
                    "typeTexts": [test_type],
                }
            }
        }))
        .unwrap()
    }

    #[test]
    fn reports_always_truthy_object_condition() {
        let diagnostics = run(&if_node("Identifier", "object"));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "alwaysTruthy");
        assert_eq!(
            diagnostics[0].range,
            super::super::super::TextRange::new(4, 10)
        );
    }

    #[test]
    fn reports_always_falsy_condition() {
        let diagnostics = run(&if_node("Identifier", "undefined"));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "alwaysFalsy");
    }

    #[test]
    fn reports_never_condition() {
        let diagnostics = run(&if_node("Identifier", "never"));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "never");
    }

    #[test]
    fn does_not_report_boolean_condition() {
        let diagnostics = run(&if_node("Identifier", "boolean"));
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_report_union_with_object_and_nullish() {
        // `null | object` is neither always truthy nor always falsy.
        let diagnostics = run(&if_node("Identifier", "null | object"));
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn unary_not_flips_truthiness() {
        // `!alwaysTruthy` is always falsy.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "IfStatement",
            "range": { "start": 0, "end": 20 },
            "children": {
                "test": {
                    "kind": "UnaryExpression",
                    "range": { "start": 4, "end": 11 },
                    "fields": { "operator": "!" },
                    "children": {
                        "argument": {
                            "kind": "Identifier",
                            "range": { "start": 5, "end": 11 },
                            "typeTexts": ["object"],
                        }
                    }
                }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "alwaysFalsy");
    }

    #[test]
    fn reports_never_nullish_on_nullish_coalescing() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "LogicalExpression",
            "range": { "start": 0, "end": 12 },
            "fields": { "operator": "??" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 3 },
                    "typeTexts": ["string"],
                },
                "right": {
                    "kind": "Literal",
                    "range": { "start": 7, "end": 12 },
                    "typeTexts": ["\"x\""],
                }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "neverNullish");
    }

    #[test]
    fn does_not_report_nullable_nullish_coalescing() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "LogicalExpression",
            "range": { "start": 0, "end": 12 },
            "fields": { "operator": "??" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 3 },
                    "typeTexts": ["string | undefined"],
                },
                "right": {
                    "kind": "Literal",
                    "range": { "start": 7, "end": 12 },
                    "typeTexts": ["\"x\""],
                }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_comparison_between_literal_types() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "operator": "===" },
            "children": {
                "left": {
                    "kind": "Literal",
                    "range": { "start": 0, "end": 1 },
                    "typeTexts": ["1"],
                },
                "right": {
                    "kind": "Literal",
                    "range": { "start": 6, "end": 7 },
                    "typeTexts": ["2"],
                }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "comparisonBetweenLiteralTypes");
    }

    #[test]
    fn does_not_report_comparison_with_non_literal() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "operator": "===" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 1 },
                    "typeTexts": ["number"],
                },
                "right": {
                    "kind": "Literal",
                    "range": { "start": 6, "end": 7 },
                    "typeTexts": ["2"],
                }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }
}

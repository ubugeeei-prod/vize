use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::{rule_option_bool, strip_chain_expression};
use crate::utils::{TypeTextKind, classify_type_text, split_type_text};

/// Type-aware rule that flags unnecessary comparisons against boolean literals.
///
/// Faithful port of the tsgolint `no-unnecessary-boolean-literal-compare` rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryBooleanLiteralCompareRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "comparingNullableToFalse",
        description: "This expression unnecessarily compares a nullable boolean value to false instead of using the ?? operator to provide a default.",
    },
    RuleMessage {
        id: "comparingNullableToTrueDirect",
        description: "This expression unnecessarily compares a nullable boolean value to true instead of using it directly.",
    },
    RuleMessage {
        id: "comparingNullableToTrueNegated",
        description: "This expression unnecessarily compares a nullable boolean value to true instead of negating it.",
    },
    RuleMessage {
        id: "direct",
        description: "This expression unnecessarily compares a boolean value to a boolean instead of using it directly.",
    },
    RuleMessage {
        id: "negated",
        description: "This expression unnecessarily compares a boolean value to a boolean instead of negating it.",
    },
    RuleMessage {
        id: "noStrictNullCheck",
        description: "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    },
];

// Listen to `Program` as well so the one-off `noStrictNullCheck` diagnostic can
// be emitted once per file (matching the Go rule, which reports it before
// traversal begins, independent of whether any `BinaryExpression` exists).
const LISTENERS: &[&str] = &["Program", "BinaryExpression"];

impl RustLintRule for NoUnnecessaryBooleanLiteralCompareRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-boolean-literal-compare"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow unnecessary comparisons against boolean literals."
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
        // The Go rule emits a one-off `noStrictNullCheck` diagnostic at range
        // (0, 0) when strictNullChecks is disabled, BEFORE traversal and
        // independent of any `BinaryExpression`. The host attaches the
        // `__strictNullChecks` raw fact (false when the compiler option is off)
        // to the `Program` node, so we emit it exactly once there.
        if node.kind == "Program" {
            if node.field_bool("__strictNullChecks") == Some(false) {
                ctx.report("noStrictNullCheck", TextRange::new(0, 0));
            }
            return;
        }

        if node.kind != "BinaryExpression" {
            return;
        }

        let options = Options::from_node(node);

        let Some(comparison) = boolean_comparison(node) else {
            return;
        };

        // Honor the allow-* options for nullable boolean comparisons.
        if comparison.expression_is_nullable_boolean
            && ((comparison.literal_boolean_in_comparison
                && options.allow_comparing_nullable_booleans_to_true)
                || (!comparison.literal_boolean_in_comparison
                    && options.allow_comparing_nullable_booleans_to_false))
        {
            return;
        }

        let message_id = if comparison.expression_is_nullable_boolean {
            if comparison.literal_boolean_in_comparison {
                if comparison.negated {
                    "comparingNullableToTrueNegated"
                } else {
                    "comparingNullableToTrueDirect"
                }
            } else {
                "comparingNullableToFalse"
            }
        } else if comparison.negated {
            "negated"
        } else {
            "direct"
        };

        let suggestions = build_suggestions(node, &comparison, message_id);
        ctx.report_with_suggestions(message_id, node.range, suggestions);
    }
}

#[derive(Clone, Copy, Debug)]
struct Options {
    allow_comparing_nullable_booleans_to_false: bool,
    allow_comparing_nullable_booleans_to_true: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        // Both default to true, matching the upstream schema defaults.
        Self {
            allow_comparing_nullable_booleans_to_false: rule_option_bool(
                node,
                "allowComparingNullableBooleansToFalse",
            )
            .unwrap_or(true),
            allow_comparing_nullable_booleans_to_true: rule_option_bool(
                node,
                "allowComparingNullableBooleansToTrue",
            )
            .unwrap_or(true),
        }
    }
}

/// Mirror of the Go `booleanComparison` struct, holding which side carried the
/// literal, whether the comparison was negated, and the operand expression.
struct BooleanComparison<'a> {
    /// Operand expression that is being compared against a boolean literal.
    expression: &'a LintNode,
    /// The literal compared against was `true` (vs `false`).
    literal_boolean_in_comparison: bool,
    /// The operator was `!==` / `!=`.
    negated: bool,
    /// The operand's constraint type is a nullable-boolean union.
    expression_is_nullable_boolean: bool,
}

/// Reproduces `getBooleanComparison` from the Go source.
fn boolean_comparison(node: &LintNode) -> Option<BooleanComparison<'_>> {
    let operator = node.field_str("operator")?;
    let negated = match operator {
        "!==" | "!=" => true,
        "===" | "==" => false,
        _ => return None,
    };

    let left = node.child("left")?;
    let right = node.child("right")?;

    // `deconstruct(against, expression)`: the `against` side must be a boolean
    // literal; the other side becomes the compared expression.
    let (expression, literal_boolean_in_comparison) =
        match boolean_literal_kind(strip_parens(right)) {
            Some(is_true) => (left, is_true),
            None => (right, boolean_literal_kind(strip_parens(left))?),
        };

    // Inspect the constraint type of the compared expression.
    let info = ConstraintInfo::of(expression);
    if info.is_unconstrained_type_parameter {
        return None;
    }

    if info.is_boolean {
        return Some(BooleanComparison {
            expression,
            literal_boolean_in_comparison,
            negated,
            expression_is_nullable_boolean: false,
        });
    }

    if info.is_nullable_boolean {
        return Some(BooleanComparison {
            expression,
            literal_boolean_in_comparison,
            negated,
            expression_is_nullable_boolean: true,
        });
    }

    None
}

/// Returns `Some(true)` for a `true` literal, `Some(false)` for a `false`
/// literal, and `None` otherwise. Mirrors the `TrueKeyword`/`FalseKeyword`
/// checks in the Go `deconstruct` closure.
fn boolean_literal_kind(node: &LintNode) -> Option<bool> {
    let current = strip_chain_expression(node);
    match current
        .fields
        .get("value")
        .and_then(|value| value.as_bool())
    {
        Some(literal) if current.kind == "Literal" => Some(literal),
        _ => None,
    }
}

/// Removes wrapping `TSNonNullExpression`-free parentheses. TS-ESTree does not
/// emit explicit parenthesized nodes, but a host may surface them; strip both
/// the chain wrapper and an explicit `ParenthesizedExpression` if present.
fn strip_parens(node: &LintNode) -> &LintNode {
    let mut current = strip_chain_expression(node);
    while current.kind == "ParenthesizedExpression" || current.kind == "TSParenthesizedType" {
        match current.child("expression") {
            Some(inner) => current = strip_chain_expression(inner),
            None => break,
        }
    }
    current
}

/// Constraint-type facts about a compared expression. The host provides the
/// raw constraint constituents (`__constraintTypeTexts`) and a flag marking an
/// unconstrained type parameter (`__isUnconstrainedTypeParameter`). When those
/// raw facts are absent the rule falls back to the rendered `type_texts`.
struct ConstraintInfo {
    is_unconstrained_type_parameter: bool,
    is_boolean: bool,
    is_nullable_boolean: bool,
}

impl ConstraintInfo {
    fn of(node: &LintNode) -> Self {
        let target = strip_chain_expression(node);

        let is_unconstrained_type_parameter =
            target.field_bool("__isUnconstrainedTypeParameter") == Some(true);

        // Prefer the host-provided constraint constituents (a raw checker query
        // result). These are already split into top-level union members.
        let constituents: Vec<String> = match target
            .fields
            .get("__constraintTypeTexts")
            .and_then(|value| value.as_array())
        {
            Some(items) => items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect(),
            None => {
                // Fall back to the rendered type texts, splitting each into its
                // top-level union/intersection constituents.
                target
                    .type_texts
                    .iter()
                    .flat_map(|text| split_type_text(text))
                    .collect()
            }
        };

        let is_boolean = is_boolean_constraint(&constituents);
        let is_nullable_boolean = is_nullable_boolean_constraint(&constituents);

        Self {
            is_unconstrained_type_parameter,
            is_boolean,
            is_nullable_boolean,
        }
    }
}

/// `isBooleanType`: the constraint is exactly a boolean-like type (one or more
/// of `boolean` / `true` / `false`, and nothing else).
fn is_boolean_constraint(constituents: &[String]) -> bool {
    !constituents.is_empty()
        && constituents
            .iter()
            .all(|part| classify_type_text(Some(part.trim())) == TypeTextKind::Boolean)
}

/// `isNullableBoolean`: a union that contains at least one nullish constituent
/// AND at least one boolean-like constituent. Matching the Go source, the union
/// may also contain unrelated members (e.g. `true | string | undefined`); those
/// do NOT disqualify it. The presence of other members only changes whether the
/// `allow*` option short-circuit is reached, never the classification itself.
fn is_nullable_boolean_constraint(constituents: &[String]) -> bool {
    // Mirror `utils.IsUnionType(t)`: a non-union operand is never a nullable
    // boolean. Splitting always yields a single constituent for a non-union
    // rendered type, so require at least two.
    if constituents.len() < 2 {
        return false;
    }
    let mut has_nullish = false;
    let mut has_boolean = false;
    for part in constituents {
        match classify_type_text(Some(part.trim())) {
            TypeTextKind::Nullish => has_nullish = true,
            TypeTextKind::Boolean => has_boolean = true,
            // Other members (string, object, etc.) are tolerated, matching the
            // Go `flags&Nullable != 0 && flags&BooleanLike != 0` check.
            _ => {}
        }
    }
    has_nullish && has_boolean
}

/// Builds the autofix suggestion, mirroring `ReportNodeWithFixes` in Go.
fn build_suggestions(
    node: &LintNode,
    comparison: &BooleanComparison<'_>,
    message_id: &'static str,
) -> Vec<LintSuggestion> {
    // We can only build a textual fix when we know the source slice of the
    // compared expression. The host attaches `text` on operand nodes.
    let Some(expression_text) = comparison.expression.text.clone() else {
        return Vec::new();
    };

    // Determine whether the (paren-skipped) parent is a unary `!`. The host
    // exposes the paren-skipped parent kind via `__parentKindSkippingParens`
    // (raw fact); fall back to `__parentKind`.
    let parent_is_unary_negation = node.field_str("__parentKindSkippingParens")
        == Some("unary-negation")
        || (node.field_str("__parentKind") == Some("UnaryExpression")
            && node.field_str("__parentUnaryOperator") == Some("!"));

    // When the parent is a unary negation, the replaced range extends over the
    // parent node, which the host exposes via `__unaryParentRange`.
    let mutated_range = if parent_is_unary_negation {
        unary_parent_range(node).unwrap_or(node.range)
    } else {
        node.range
    };

    let should_negate = comparison.negated != comparison.literal_boolean_in_comparison;

    // Reproduce the Go fix list, which appends edits in this order and lets the
    // editor stack same-position inserts in insertion order:
    //   1. replace mutated node with the operand text
    //   2. when `shouldNegate == isUnaryNegation`: prefix `!`, then (when the
    //      operand is not a strong-precedence node) wrap in `( ... )`
    //   3. for nullable-to-false: wrap in `( ... ?? true)`
    // Negation parens are the inner layer; the nullish parens are the outer
    // layer, so a non-strong nullable-to-false operand becomes `((expr) ?? true)`.
    let needs_negation_prefix = should_negate == parent_is_unary_negation;
    let needs_negation_parens =
        needs_negation_prefix && !is_strong_precedence_node(comparison.expression);
    let needs_nullish_default =
        comparison.expression_is_nullable_boolean && !comparison.literal_boolean_in_comparison;

    let mut replacement = String::new();
    if needs_negation_prefix {
        replacement.push('!');
    }
    // Outer nullish paren.
    if needs_nullish_default {
        replacement.push('(');
    }
    // Inner negation paren.
    if needs_negation_parens {
        replacement.push('(');
    }
    replacement.push_str(&expression_text);
    if needs_negation_parens {
        replacement.push(')');
    }
    if needs_nullish_default {
        replacement.push_str(" ?? true)");
    }

    vec![LintSuggestion {
        message_id: message_id.to_owned(),
        message: message_for(message_id).to_owned(),
        fixes: vec![LintFix::replace_range(mutated_range, replacement)],
    }]
}

fn unary_parent_range(node: &LintNode) -> Option<TextRange> {
    let start = node.field_f64("__unaryParentStart")?;
    let end = node.field_f64("__unaryParentEnd")?;
    Some(TextRange::new(start as u32, end as u32))
}

/// `utils.IsStrongPrecedenceNode`: nodes that bind tightly enough not to need
/// wrapping parentheses when prefixed with `!`. The ESTree kinds below mirror
/// the TS AST kinds enumerated in the Go helper:
///   - LiteralKind / BooleanLiteral        -> `Literal`
///   - ParenthesizedExpression             -> `ParenthesizedExpression`
///   - Identifier                          -> `Identifier`
///   - TypeReference                       -> `TSTypeReference`
///   - TypeOperator                        -> `TSTypeOperator`
///   - ArrayLiteralExpression              -> `ArrayExpression`
///   - ObjectLiteralExpression             -> `ObjectExpression`
///   - PropertyAccess / ElementAccess      -> `MemberExpression`
///   - CallExpression                      -> `CallExpression`
///   - NewExpression                       -> `NewExpression`
///   - TaggedTemplateExpression            -> `TaggedTemplateExpression`
///   - ExpressionWithTypeArguments         -> `TSInstantiationExpression`
///
/// Note: `UnaryExpression`, `ThisExpression`, plain `TemplateLiteral` and
/// `TSNonNullExpression` are deliberately NOT included, matching the Go list.
fn is_strong_precedence_node(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    matches!(
        current.kind.as_str(),
        "Identifier"
            | "Literal"
            | "MemberExpression"
            | "CallExpression"
            | "NewExpression"
            | "TaggedTemplateExpression"
            | "ArrayExpression"
            | "ObjectExpression"
            | "ParenthesizedExpression"
            | "TSTypeReference"
            | "TSTypeOperator"
            | "TSInstantiationExpression"
    )
}

fn message_for(message_id: &str) -> &'static str {
    MESSAGES
        .iter()
        .find(|message| message.id == message_id)
        .map(|message| message.description)
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value, json};

    use super::super::super::{LintDiagnostic, LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnnecessaryBooleanLiteralCompareRule;

    fn run(node: &LintNode) -> Vec<LintDiagnostic> {
        let rule = NoUnnecessaryBooleanLiteralCompareRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn node(kind: &str, range: TextRange) -> LintNode {
        LintNode {
            kind: kind.to_owned(),
            range,
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    fn bool_literal(value: bool, range: TextRange) -> LintNode {
        let mut literal = node("Literal", range);
        literal.fields.insert("value".to_owned(), json!(value));
        literal.text = Some(value.to_string());
        literal
    }

    fn ident(name: &str, type_text: &str, range: TextRange) -> LintNode {
        let mut identifier = node("Identifier", range);
        identifier.fields.insert("name".to_owned(), json!(name));
        identifier.text = Some(name.to_owned());
        if !type_text.is_empty() {
            identifier.type_texts = vec![type_text.to_owned()];
        }
        identifier
    }

    fn binary(operator: &str, left: LintNode, right: LintNode) -> LintNode {
        let mut bin = node("BinaryExpression", TextRange::new(0, 32));
        bin.fields
            .insert("operator".to_owned(), Value::String(operator.to_owned()));
        bin.children.insert("left".to_owned(), left);
        bin.children.insert("right".to_owned(), right);
        bin
    }

    // ---- should report ----

    #[test]
    fn reports_direct_boolean_compare() {
        // `varBoolean === true;`
        let bin = binary(
            "===",
            ident("varBoolean", "boolean", TextRange::new(0, 10)),
            bool_literal(true, TextRange::new(15, 19)),
        );
        let diagnostics = run(&bin);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "direct");
    }

    #[test]
    fn reports_negated_boolean_compare() {
        // `varBoolean !== false;`
        let bin = binary(
            "!==",
            ident("varBoolean", "boolean", TextRange::new(0, 10)),
            bool_literal(false, TextRange::new(15, 20)),
        );
        let diagnostics = run(&bin);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "negated");
    }

    #[test]
    fn reports_nullable_to_false_when_option_disabled() {
        // `varBooleanOrNull === false;` with allowComparingNullableBooleansToFalse: false
        let mut bin = binary(
            "===",
            ident("varBooleanOrNull", "boolean | null", TextRange::new(0, 16)),
            bool_literal(false, TextRange::new(20, 25)),
        );
        bin.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowComparingNullableBooleansToFalse": false }]),
        );
        let diagnostics = run(&bin);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "comparingNullableToFalse");
    }

    #[test]
    fn reports_nullable_to_true_direct_when_option_disabled() {
        // `varTrueOrUndefined === true;` with allowComparingNullableBooleansToTrue: false
        let mut bin = binary(
            "===",
            ident(
                "varTrueOrUndefined",
                "true | undefined",
                TextRange::new(0, 18),
            ),
            bool_literal(true, TextRange::new(22, 26)),
        );
        bin.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowComparingNullableBooleansToTrue": false }]),
        );
        let diagnostics = run(&bin);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "comparingNullableToTrueDirect");
    }

    // ---- should NOT report ----

    #[test]
    fn ignores_non_boolean_operand() {
        // `varString === false;`
        let bin = binary(
            "===",
            ident("varString", "string", TextRange::new(0, 9)),
            bool_literal(false, TextRange::new(14, 19)),
        );
        assert!(run(&bin).is_empty());
    }

    #[test]
    fn ignores_nullable_to_true_by_default() {
        // `varBooleanOrUndefined === true;` (default allows nullable-to-true)
        let bin = binary(
            "===",
            ident(
                "varBooleanOrUndefined",
                "boolean | undefined",
                TextRange::new(0, 21),
            ),
            bool_literal(true, TextRange::new(25, 29)),
        );
        assert!(run(&bin).is_empty());
    }

    #[test]
    fn ignores_unconstrained_type_parameter() {
        // `someCondition === true;` where someCondition: T (no constraint)
        let mut operand = ident("someCondition", "T", TextRange::new(0, 13));
        operand
            .fields
            .insert("__isUnconstrainedTypeParameter".to_owned(), json!(true));
        let bin = binary("===", operand, bool_literal(true, TextRange::new(18, 22)));
        assert!(run(&bin).is_empty());
    }

    #[test]
    fn ignores_boolean_or_string_union() {
        // `varBooleanOrString === false;` (union has non-boolean, non-nullish)
        let bin = binary(
            "===",
            ident(
                "varBooleanOrString",
                "boolean | string",
                TextRange::new(0, 18),
            ),
            bool_literal(false, TextRange::new(23, 28)),
        );
        assert!(run(&bin).is_empty());
    }

    #[test]
    fn warns_once_on_program_when_strict_null_checks_disabled() {
        // The Go rule emits `noStrictNullCheck` once, globally, even when there
        // is no comparison in the file. We mirror that on the `Program` node.
        let mut program = node("Program", TextRange::new(0, 40));
        program
            .fields
            .insert("__strictNullChecks".to_owned(), json!(false));
        let diagnostics = run(&program);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noStrictNullCheck");
        assert_eq!(diagnostics[0].range, TextRange::new(0, 0));
    }

    #[test]
    fn does_not_warn_on_program_when_strict_null_checks_enabled() {
        let mut program = node("Program", TextRange::new(0, 40));
        program
            .fields
            .insert("__strictNullChecks".to_owned(), json!(true));
        assert!(run(&program).is_empty());
    }

    #[test]
    fn binary_node_does_not_emit_strict_null_check_warning() {
        // The flag is only honored on the `Program` node, so a stray flag on a
        // comparison must not produce a duplicate diagnostic.
        let mut bin = binary(
            "===",
            ident("varString", "string", TextRange::new(0, 9)),
            bool_literal(false, TextRange::new(14, 19)),
        );
        bin.fields
            .insert("__strictNullChecks".to_owned(), json!(false));
        assert!(run(&bin).is_empty());
    }
}

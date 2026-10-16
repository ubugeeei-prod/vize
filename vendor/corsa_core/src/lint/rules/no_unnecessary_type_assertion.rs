use serde_json::Value;

use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};

/// Type-aware rule that disallows type assertions that do not change the type
/// of the expression.
///
/// This is a faithful port of the tsgolint `no-unnecessary-type-assertion`
/// Go rule. The host AST traversal and the TypeScript checker queries live on
/// the JavaScript bridge; the rule decision logic lives here in Rust and
/// operates on `LintNode` facts.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryTypeAssertionRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unnecessaryAssertion",
        description: "This assertion is unnecessary since it does not change the type of the expression.",
    },
    RuleMessage {
        id: "contextuallyUnnecessary",
        description: "This assertion is unnecessary since the receiver accepts the original type of the expression.",
    },
];

// `TSAsExpression`/`TSTypeAssertion` cover `x as T` and `<T>x`.
// `TSNonNullExpression` covers `x!`.
const LISTENERS: &[&str] = &["TSAsExpression", "TSTypeAssertion", "TSNonNullExpression"];

impl RustLintRule for NoUnnecessaryTypeAssertionRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-assertion"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow type assertions that do not change the expression type."
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
        let options = Options::from_node(node);
        match node.kind.as_str() {
            "TSAsExpression" | "TSTypeAssertion" => check_type_assertion(ctx, node, &options),
            "TSNonNullExpression" => check_non_null_assertion(ctx, node),
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Options {
    check_literal_const_assertions: bool,
    types_to_ignore: Vec<String>,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        let first = node
            .fields
            .get("__ruleOptions")
            .and_then(Value::as_array)
            .and_then(|items| items.first());
        let check_literal_const_assertions = first
            .and_then(|value| value.get("checkLiteralConstAssertions"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let types_to_ignore = first
            .and_then(|value| value.get("typesToIgnore"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            check_literal_const_assertions,
            types_to_ignore,
        }
    }
}

// ---------------------------------------------------------------------------
// `as T` / `<T>x` assertions
// ---------------------------------------------------------------------------

fn check_type_assertion(ctx: &mut RuleContext<'_>, node: &LintNode, options: &Options) {
    if node.child("expression").is_none() {
        return;
    }

    // `slices.Contains(opts.TypesToIgnore, trimmed source text of the type node)`.
    // The host renders the asserted type's source text into `__typeAnnotationText`.
    if let Some(annotation_text) = node.field_str("__typeAnnotationText") {
        let trimmed = annotation_text.trim();
        if options
            .types_to_ignore
            .iter()
            .any(|ignored| ignored.trim() == trimmed)
        {
            return;
        }
    }

    // `isConstAssertion(typeNode)` -> a `TypeReference` named `const`.
    let type_annotation_is_const_assertion = is_const_assertion(node);

    // `isTypeLiteral(castType)` -> the cast type is a string/number/bigint/boolean literal.
    // Provided as a raw checker fact: whether `GetTypeAtLocation(node)` is a unit literal type.
    let cast_type_is_literal = node.field_bool("__castTypeIsLiteral").unwrap_or(false);

    // `if !opts.CheckLiteralConstAssertions && castTypeIsLiteral && typeAnnotationIsConstAssertion { return }`
    if !options.check_literal_const_assertions
        && cast_type_is_literal
        && type_annotation_is_const_assertion
    {
        return;
    }

    // `typeIsUnchanged := isTypeUnchanged(uncastType, castType)`.
    // The host computes the structural comparison from the Go `isTypeUnchanged`
    // algorithm (identity, plus the exactOptionalPropertyTypes undefined-union
    // special case) and exposes the boolean verdict's *input* as a raw fact:
    // whether the checker's uncast and cast types are considered unchanged.
    let type_is_unchanged = node.field_bool("__typeIsUnchanged").unwrap_or(false);

    // `wouldSameTypeBeInferred`.
    let would_same_type_be_inferred = if cast_type_is_literal {
        is_implicitly_narrowed_literal_declaration(node)
    } else {
        !type_annotation_is_const_assertion
    };

    if !type_is_unchanged || !would_same_type_be_inferred {
        return;
    }

    // The Go rule reports the `as T` / `<T>` portion only and offers a fix that
    // removes it. The host provides the precise assertion range (the `as T`
    // tokens for `TSAsExpression`, or the `<T>` tokens for `TSTypeAssertion`)
    // via `__assertionRange`; fall back to the whole-node range.
    let assertion_range = assertion_range(node);
    let fix_range = remove_range(node).unwrap_or(assertion_range);

    let suggestion = LintSuggestion {
        message_id: "unnecessaryAssertion".to_owned(),
        message: MESSAGES[0].description.to_owned(),
        fixes: vec![LintFix::remove_range(fix_range)],
    };
    ctx.report_with_suggestions("unnecessaryAssertion", assertion_range, vec![suggestion]);
}

fn is_const_assertion(node: &LintNode) -> bool {
    // ESTree: a `const` assertion has a `typeAnnotation` that is a
    // `TSTypeReference` whose `typeName` is the identifier `const`.
    let Some(annotation) = node.child("typeAnnotation") else {
        // Fall back to the rendered annotation text.
        return node.field_str("__typeAnnotationText").map(str::trim) == Some("const");
    };
    if annotation.kind != "TSTypeReference" {
        return false;
    }
    annotation
        .child("typeName")
        .and_then(|type_name| {
            if type_name.kind == "Identifier" {
                type_name.field_stringish("name")
            } else {
                None
            }
        })
        .as_deref()
        == Some("const")
}

fn is_implicitly_narrowed_literal_declaration(node: &LintNode) -> bool {
    // Template literals with expressions can be widened, so a `const` assertion
    // on them is meaningful: never treat them as implicitly narrowed.
    if node.child("expression").is_some_and(|expression| {
        expression.kind == "TemplateLiteral" && has_template_expressions(expression)
    }) {
        return false;
    }

    // The remaining condition requires parent context (a `const` variable
    // declaration, or a `readonly` property declaration). The host exposes this
    // as a raw fact derived from the parent AST nodes.
    node.field_bool("__implicitlyNarrowedLiteralDeclaration")
        .unwrap_or(false)
}

fn has_template_expressions(node: &LintNode) -> bool {
    node.child_list("expressions")
        .map(|expressions| !expressions.is_empty())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// `x!` non-null assertions
// ---------------------------------------------------------------------------

fn check_non_null_assertion(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let Some(expression) = node.child("expression") else {
        return;
    };

    // `if ast.IsAssignmentExpression(node.Parent, true)`.
    if node.field_str("__parentKind") == Some("AssignmentExpression") {
        // Only report when the non-null assertion is the assignment target
        // (the left-hand side). The host exposes this as a raw positional fact.
        if node.field_bool("__isAssignmentLeft").unwrap_or(false) {
            let range = exclamation_range(node);
            let suggestion = remove_exclamation_suggestion("contextuallyUnnecessary", range);
            ctx.report_with_suggestions("contextuallyUnnecessary", range, vec![suggestion]);
        }
        // For all other `=` assignments we ignore the non-null assertion, since
        // it can change the type-flow of following code.
        return;
    }

    // `t := GetConstrainedTypeAtLocation(expression)`; gather union part flags.
    let flags = TypeFlags::of_expression(node);

    // `if tFlags & (Any|Unknown|Null|Undefined|Void) == 0`.
    if !flags.is_nullable_or_top() {
        // The expression is already non-nullable -> the `!` is unnecessary.
        // `if ast.IsIdentifier(expression) && isPossiblyUsedBeforeAssigned(expression) { return }`.
        if expression.kind == "Identifier"
            && node
                .field_bool("__possiblyUsedBeforeAssigned")
                .unwrap_or(false)
        {
            return;
        }
        let range = exclamation_range(node);
        let suggestion = remove_exclamation_suggestion("unnecessaryAssertion", range);
        ctx.report_with_suggestions("unnecessaryAssertion", range, vec![suggestion]);
        return;
    }

    // The expression is nullable: report only if used in a position whose
    // contextual type already accepts the same nullable members.
    let Some(contextual) = TypeFlags::of_contextual(node) else {
        return;
    };

    // `if tFlags&Unknown != 0 && contextualFlags&Unknown == 0 { return }`.
    if flags.unknown && !contextual.unknown {
        return;
    }

    // In strict mode you can't assign `null` to `undefined`, so the parent must
    // accept every nullable member that the asserted type contributes.
    let is_valid_undefined = !flags.undefined || contextual.undefined;
    let is_valid_null = !flags.null || contextual.null;
    let is_valid_void = !flags.void_ || contextual.void_;

    if is_valid_undefined && is_valid_null && is_valid_void {
        let range = exclamation_range(node);
        let suggestion = remove_exclamation_suggestion("contextuallyUnnecessary", range);
        ctx.report_with_suggestions("contextuallyUnnecessary", range, vec![suggestion]);
    }
}

/// Collected TypeScript `TypeFlags` membership for the union parts of a type.
///
/// These mirror the Go rule's `tFlags`/`contextualFlags` bitsets. The host
/// computes the raw flag set per the checker and exposes them as booleans so
/// the verdict stays in Rust.
#[derive(Clone, Copy, Debug, Default)]
struct TypeFlags {
    any: bool,
    unknown: bool,
    null: bool,
    undefined: bool,
    void_: bool,
}

impl TypeFlags {
    fn of_expression(node: &LintNode) -> Self {
        Self::read(node, "__expressionTypeFlags")
    }

    fn of_contextual(node: &LintNode) -> Option<Self> {
        // `GetContextualType` returns nil when there is no contextual type.
        node.fields.get("__contextualTypeFlags")?;
        Some(Self::read(node, "__contextualTypeFlags"))
    }

    fn read(node: &LintNode, key: &str) -> Self {
        let flags = node
            .fields
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let has = |name: &str| flags.iter().any(|flag| flag == name);
        Self {
            any: has("any"),
            unknown: has("unknown"),
            null: has("null"),
            undefined: has("undefined"),
            void_: has("void"),
        }
    }

    fn is_nullable_or_top(self) -> bool {
        self.any || self.unknown || self.null || self.undefined || self.void_
    }
}

// ---------------------------------------------------------------------------
// Range helpers
// ---------------------------------------------------------------------------

fn assertion_range(node: &LintNode) -> TextRange {
    read_range_field(node, "__assertionRange").unwrap_or(node.range)
}

fn remove_range(node: &LintNode) -> Option<TextRange> {
    read_range_field(node, "__removeRange")
}

fn exclamation_range(node: &LintNode) -> TextRange {
    // The host scans for the `!` token after the expression and exposes its
    // range; fall back to the trailing byte of the node.
    read_range_field(node, "__exclamationRange").unwrap_or_else(|| {
        let end = node.range.end;
        TextRange::new(end.saturating_sub(1), end)
    })
}

fn read_range_field(node: &LintNode, key: &str) -> Option<TextRange> {
    let value = node.fields.get(key)?;
    let start = value.get("start").and_then(Value::as_u64)? as u32;
    let end = value.get("end").and_then(Value::as_u64)? as u32;
    Some(TextRange::new(start, end))
}

fn remove_exclamation_suggestion(message_id: &str, range: TextRange) -> LintSuggestion {
    let message = MESSAGES
        .iter()
        .find(|message| message.id == message_id)
        .map(|message| message.description)
        .unwrap_or(message_id)
        .to_owned();
    LintSuggestion {
        message_id: message_id.to_owned(),
        message,
        fixes: vec![LintFix::remove_range(range)],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnnecessaryTypeAssertionRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        // NOTE: `super::super` is `crate::lint` (where RuleContext lives) when this
        // file is placed at `src/core/corsa_core/src/lint/rules/<snake>.rs`.
        let rule = NoUnnecessaryTypeAssertionRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn lint_node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode json")
    }

    // ---- should report ----

    #[test]
    fn reports_literal_as_same_literal() {
        // `const foo = 3 as 3;` -> unnecessary.
        let node = lint_node(json!({
            "kind": "TSAsExpression",
            "range": { "start": 12, "end": 18 },
            "fields": {
                "__typeAnnotationText": "3",
                "__castTypeIsLiteral": true,
                "__typeIsUnchanged": true,
                "__implicitlyNarrowedLiteralDeclaration": true,
                "__assertionRange": { "start": 14, "end": 18 }
            },
            "children": {
                "expression": { "kind": "Literal", "range": { "start": 12, "end": 13 } }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unnecessaryAssertion");
        assert_eq!(diagnostics[0].range, TextRange::new(14, 18));
    }

    #[test]
    fn reports_unchanged_non_literal_as() {
        // `const redundant = str as string;` -> unnecessary.
        let node = lint_node(json!({
            "kind": "TSAsExpression",
            "range": { "start": 19, "end": 32 },
            "fields": {
                "__typeAnnotationText": "string",
                "__castTypeIsLiteral": false,
                "__typeIsUnchanged": true
            },
            "children": {
                "expression": {
                    "kind": "Identifier",
                    "range": { "start": 19, "end": 22 },
                    "typeTexts": ["string"]
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unnecessaryAssertion");
    }

    #[test]
    fn reports_unnecessary_non_null_on_non_nullable() {
        // `let bar: number; bar! + 1;` -> the `!` is unnecessary.
        let node = lint_node(json!({
            "kind": "TSNonNullExpression",
            "range": { "start": 0, "end": 4 },
            "fields": {
                "__parentKind": "BinaryExpression",
                "__expressionTypeFlags": ["number"],
                "__exclamationRange": { "start": 3, "end": 4 }
            },
            "children": {
                "expression": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 3 },
                    "typeTexts": ["number"]
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unnecessaryAssertion");
        assert_eq!(diagnostics[0].range, TextRange::new(3, 4));
    }

    #[test]
    fn reports_contextually_unnecessary_non_null_assignment_target() {
        // `x! = 1;` where `x` is the assignment target.
        let node = lint_node(json!({
            "kind": "TSNonNullExpression",
            "range": { "start": 0, "end": 2 },
            "fields": {
                "__parentKind": "AssignmentExpression",
                "__isAssignmentLeft": true,
                "__exclamationRange": { "start": 1, "end": 2 }
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 0, "end": 1 } }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "contextuallyUnnecessary");
    }

    // ---- should NOT report ----

    #[test]
    fn ignores_literal_const_assertion_by_default() {
        // `let z = c as const;` -> not reported unless checkLiteralConstAssertions.
        let node = lint_node(json!({
            "kind": "TSAsExpression",
            "range": { "start": 8, "end": 18 },
            "fields": {
                "__typeAnnotationText": "const",
                "__castTypeIsLiteral": true,
                "__typeIsUnchanged": true,
                "__implicitlyNarrowedLiteralDeclaration": true
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 8, "end": 9 } },
                "typeAnnotation": {
                    "kind": "TSTypeReference",
                    "range": { "start": 13, "end": 18 },
                    "children": {
                        "typeName": {
                            "kind": "Identifier",
                            "range": { "start": 13, "end": 18 },
                            "fields": { "name": "const" }
                        }
                    }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_type_in_types_to_ignore() {
        // `const foo = (3 + 5) as any;` with typesToIgnore: ["any"].
        let node = lint_node(json!({
            "kind": "TSAsExpression",
            "range": { "start": 12, "end": 26 },
            "fields": {
                "__typeAnnotationText": "any",
                "__castTypeIsLiteral": false,
                "__typeIsUnchanged": true,
                "__ruleOptions": [ { "typesToIgnore": ["any"] } ]
            },
            "children": {
                "expression": { "kind": "BinaryExpression", "range": { "start": 12, "end": 19 } }
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_changed_type_as() {
        // `let z = c as number;` where c is `1` -> the type changes (widened), keep it.
        let node = lint_node(json!({
            "kind": "TSAsExpression",
            "range": { "start": 8, "end": 19 },
            "fields": {
                "__typeAnnotationText": "number",
                "__castTypeIsLiteral": false,
                "__typeIsUnchanged": false
            },
            "children": {
                "expression": {
                    "kind": "Identifier",
                    "range": { "start": 8, "end": 9 },
                    "typeTexts": ["1"]
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_necessary_non_null_on_nullable_without_context() {
        // `let bar: number | undefined = x; let foo: number = bar!;`
        // The expression is nullable and there is no contextual fact -> keep it.
        let node = lint_node(json!({
            "kind": "TSNonNullExpression",
            "range": { "start": 0, "end": 4 },
            "fields": {
                "__parentKind": "VariableDeclarator",
                "__expressionTypeFlags": ["number", "undefined"],
                "__exclamationRange": { "start": 3, "end": 4 }
            },
            "children": {
                "expression": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 3 },
                    "typeTexts": ["number | undefined"]
                }
            }
        }));
        assert!(run(&node).is_empty());
    }
}

use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};

/// Type-aware rule that prefers a non-null assertion (`!`) over an equivalent
/// non-nullable type assertion (`as T` / `<T>expr`).
///
/// This is a faithful port of the tsgolint `non-nullable-type-assertion-style`
/// rule. The host computes the resolved union type parts of the expression and
/// of the asserted type annotation (the checker queries the Go rule performs via
/// `GetTypeAtLocation` + `UnionTypeParts`), and Rust performs the set comparison
/// and decides whether to report.
#[derive(Clone, Copy, Debug, Default)]
pub struct NonNullableTypeAssertionStyleRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "preferNonNullAssertion",
    description: "Use a ! assertion to more succinctly remove null and undefined from the type.",
}];

// ESTree / TS-ESTree node kinds matching the Go listeners
// `ast.KindAsExpression` and `ast.KindTypeAssertionExpression`.
const LISTENERS: &[&str] = &["TSAsExpression", "TSTypeAssertion"];

impl RustLintRule for NonNullableTypeAssertionStyleRule {
    fn name(&self) -> &'static str {
        "non-nullable-type-assertion-style"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer non-null assertions over equivalent non-nullable type assertions."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        // The Go rule emits an auto-fix (`ReportNodeWithFixes`). Suggestions are
        // the closest representation available on the Rust side.
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        // `ast.IsConstAssertion(node)` -> bail out for `as const`.
        if node.field_bool("__isConstAssertion").unwrap_or(false) {
            return;
        }

        let Some(expression) = node.child("expression") else {
            return;
        };

        // `getTypesIfNotLoose(expression)`: bail when the expression type is
        // any/unknown (loose).
        if node.field_bool("__expressionLoose").unwrap_or(false) {
            return;
        }
        let original_types = string_array_field(node, "__expressionUnionTypeTexts");
        if original_types.is_empty() {
            return;
        }

        // `getTypesIfNotLoose(typeAnnotation)`: bail when the asserted type is
        // any/unknown (loose).
        if node.field_bool("__assertedLoose").unwrap_or(false) {
            return;
        }
        let asserted_types = string_array_field(node, "__assertedUnionTypeTexts");
        if asserted_types.is_empty() {
            return;
        }

        // nonNullableOriginalType = original parts with nullable parts removed.
        let non_nullable_original: Vec<String> = original_types
            .iter()
            .filter(|text| !is_nullable_text(text))
            .cloned()
            .collect();

        // If nothing was removed, the original type was not nullable -> bail.
        if non_nullable_original.len() == original_types.len() {
            return;
        }

        // Per-asserted-part "could be nullable" flags (base-constraint aware on
        // the host). Parallel to `__assertedUnionTypeTexts`.
        let asserted_could_be_nullable = bool_array_field(node, "__assertedPartCouldBeNullable");

        // Each asserted part must NOT be nullable-capable, and must be present in
        // the non-nullable original set.
        for (index, asserted) in asserted_types.iter().enumerate() {
            let could_be_nullable = asserted_could_be_nullable
                .get(index)
                .copied()
                .unwrap_or_else(|| is_nullable_text(asserted));
            if could_be_nullable || !non_nullable_original.iter().any(|text| text == asserted) {
                return;
            }
        }

        // Every non-nullable original part must be present in the asserted set
        // (the two non-nullable sets must be equal).
        for original in &non_nullable_original {
            if !asserted_types.iter().any(|text| text == original) {
                return;
            }
        }

        let suggestion = build_suggestion(node, expression);
        ctx.report_with_suggestions("preferNonNullAssertion", node.range, vec![suggestion]);
    }
}

/// Builds the auto-fix-equivalent suggestion that removes the assertion syntax
/// and appends `!` to the expression.
fn build_suggestion(node: &LintNode, expression: &LintNode) -> LintSuggestion {
    let expression_range = expression.range;
    let higher_precedence = node
        .field_bool("__higherPrecedenceThanUnary")
        .unwrap_or(false);

    let mut fixes = Vec::new();

    // Go uses `ast.IsAssertionExpression(node)`, which is true for BOTH
    // `KindAsExpression` and `KindTypeAssertionExpression` (see
    // typescript-go internal/ast/utilities.go: `kind == KindTypeAssertionExpression
    // || kind == KindAsExpression`). Both of this rule's registered listeners
    // therefore take the tail-removal branch (`node.Loc.WithPos(expression.End())`),
    // so we remove from the end of the expression to the end of the node for both
    // kinds. The `else` (head-removal) branch in the Go source is unreachable for
    // the registered listeners, so we do not emulate it.
    let remove_range = TextRange::new(expression_range.end, node.range.end);
    fixes.push(LintFix::remove_range(remove_range));

    if higher_precedence {
        // Append `!` directly after the expression.
        fixes.push(LintFix::replace_range(
            TextRange::new(expression_range.end, expression_range.end),
            "!",
        ));
    } else {
        // Wrap the expression in parens and then append `!`.
        fixes.push(LintFix::replace_range(
            TextRange::new(expression_range.start, expression_range.start),
            "(",
        ));
        fixes.push(LintFix::replace_range(
            TextRange::new(expression_range.end, expression_range.end),
            ")!",
        ));
    }

    LintSuggestion {
        message_id: "preferNonNullAssertion".to_owned(),
        message: MESSAGES[0].description.to_owned(),
        fixes,
    }
}

fn string_array_field(node: &LintNode, key: &str) -> Vec<String> {
    node.fields
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn bool_array_field(node: &LintNode, key: &str) -> Vec<bool> {
    node.fields
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_bool().unwrap_or(false))
                .collect()
        })
        .unwrap_or_default()
}

/// Returns whether a single resolved type-part text is exactly a nullable type
/// (`null` or `undefined`). The host already splits union parts, so each text is
/// a leaf part rather than a union.
fn is_nullable_text(text: &str) -> bool {
    matches!(text.trim(), "null" | "undefined")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::NonNullableTypeAssertionStyleRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NonNullableTypeAssertionStyleRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    // `declare const maybe: string | undefined; const bar = maybe as string;`
    // -> reported.
    fn nullable_as_string_node() -> LintNode {
        serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 49, "end": 64 },
            "fields": {
                "__expressionUnionTypeTexts": ["string", "undefined"],
                "__assertedUnionTypeTexts": ["string"],
                "__assertedPartCouldBeNullable": [false],
                "__higherPrecedenceThanUnary": true
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 49, "end": 54 } }
            }
        }))
        .unwrap()
    }

    #[test]
    fn reports_string_or_undefined_asserted_as_string() {
        let diagnostics = run(&nullable_as_string_node());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferNonNullAssertion");
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        // expr as string -> remove " as string", insert "!" after expression.
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes[0].range.start, 54);
        assert_eq!(fixes[0].range.end, 64);
        assert_eq!(fixes[0].replacement_text, "");
        assert_eq!(fixes[1].replacement_text, "!");
    }

    #[test]
    fn reports_string_or_null_asserted_as_string() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 15 },
            "fields": {
                "__expressionUnionTypeTexts": ["string", "null"],
                "__assertedUnionTypeTexts": ["string"],
                "__assertedPartCouldBeNullable": [false],
                "__higherPrecedenceThanUnary": true
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 0, "end": 5 } }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferNonNullAssertion");
    }

    #[test]
    fn wraps_low_precedence_expression_in_parens() {
        // const b = (a || undefined) as string;  -> ( ... )!
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 24 },
            "fields": {
                "__expressionUnionTypeTexts": ["string", "undefined"],
                "__assertedUnionTypeTexts": ["string"],
                "__assertedPartCouldBeNullable": [false],
                "__higherPrecedenceThanUnary": false
            },
            "children": {
                "expression": { "kind": "LogicalExpression", "range": { "start": 1, "end": 14 } }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        let fixes = &diagnostics[0].suggestions[0].fixes;
        // remove ") as string" tail (end of expr -> end of node)
        assert_eq!(fixes[0].range.start, 14);
        assert_eq!(fixes[0].range.end, 24);
        assert_eq!(fixes[1].replacement_text, "(");
        assert_eq!(fixes[2].replacement_text, ")!");
    }

    #[test]
    fn ignores_non_nullable_original_type() {
        // declare const original: number | string; const cast = original as string;
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 15 },
            "fields": {
                "__expressionUnionTypeTexts": ["number", "string"],
                "__assertedUnionTypeTexts": ["string"],
                "__assertedPartCouldBeNullable": [false]
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 0, "end": 8 } }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_asserted_type_that_retains_null() {
        // declare const original: number | null | undefined;
        // const cast = original as number | null;  (asserted still nullable)
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 25 },
            "fields": {
                "__expressionUnionTypeTexts": ["number", "null", "undefined"],
                "__assertedUnionTypeTexts": ["number", "null"],
                "__assertedPartCouldBeNullable": [false, true]
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 0, "end": 8 } }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_loose_any_expression() {
        // declare const original: number | any; const cast = original as ...;
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 25 },
            "fields": {
                "__expressionLoose": true,
                "__assertedUnionTypeTexts": ["string", "number", "undefined"]
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 0, "end": 8 } }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    #[test]
    fn type_assertion_uses_tail_removal_like_go() {
        // `<string>maybe` where maybe: string | undefined.
        // Go's ast.IsAssertionExpression is true for TSTypeAssertion too, so the
        // remove range is the tail [expression.end, node.end) (empty here), and
        // `!` is appended after the expression.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSTypeAssertion",
            "range": { "start": 0, "end": 14 },
            "fields": {
                "__expressionUnionTypeTexts": ["string", "undefined"],
                "__assertedUnionTypeTexts": ["string"],
                "__assertedPartCouldBeNullable": [false],
                "__higherPrecedenceThanUnary": true
            },
            "children": {
                "expression": { "kind": "Identifier", "range": { "start": 8, "end": 14 } }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        let fixes = &diagnostics[0].suggestions[0].fixes;
        // Tail removal: [expression.end (14), node.end (14)) -> empty range.
        assert_eq!(fixes[0].range.start, 14);
        assert_eq!(fixes[0].range.end, 14);
        assert_eq!(fixes[0].replacement_text, "");
        assert_eq!(fixes[1].replacement_text, "!");
    }

    #[test]
    fn ignores_const_assertion() {
        // const foo = [] as const;
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSAsExpression",
            "range": { "start": 0, "end": 10 },
            "fields": { "__isConstAssertion": true },
            "children": {
                "expression": { "kind": "ArrayExpression", "range": { "start": 0, "end": 2 } }
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }
}

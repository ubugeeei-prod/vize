use serde_json::Value;

use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::rule_option_bool;
use crate::utils::{TypeTextKind, classify_type_text, split_top_level_type_text};

/// Type-aware rule that disallows `void`-typed expressions in confusing value
/// positions. Faithful port of the tsgolint `no-confusing-void-expression` rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoConfusingVoidExpressionRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "invalidVoidExpr",
        description: "Placing a void expression inside another expression is forbidden.",
    },
    RuleMessage {
        id: "invalidVoidExprArrow",
        description: "Returning a void expression from an arrow function shorthand is forbidden.",
    },
    RuleMessage {
        id: "invalidVoidExprArrowWrapVoid",
        description: "Void expressions returned from an arrow function shorthand must be marked explicitly with the `void` operator.",
    },
    RuleMessage {
        id: "invalidVoidExprReturn",
        description: "Returning a void expression from a function is forbidden. Please move it before the `return` statement.",
    },
    RuleMessage {
        id: "invalidVoidExprReturnLast",
        description: "Returning a void expression from a function is forbidden. Please remove the `return` statement.",
    },
    RuleMessage {
        id: "invalidVoidExprReturnWrapVoid",
        description: "Void expressions returned from a function must be marked explicitly with the `void` operator.",
    },
    RuleMessage {
        id: "invalidVoidExprWrapVoid",
        description: "Void expressions used inside another expression must be moved to its own statement or marked explicitly with the `void` operator.",
    },
    RuleMessage {
        id: "voidExprWrapVoid",
        description: "Mark with an explicit `void` operator.",
    },
];

// In TS-ESTree, the void-producing expressions the Go rule listens to map to:
//   ast.KindAwaitExpression          -> "AwaitExpression"
//   ast.KindCallExpression           -> "CallExpression"
//   ast.KindTaggedTemplateExpression -> "TaggedTemplateExpression"
const LISTENERS: &[&str] = &[
    "AwaitExpression",
    "CallExpression",
    "TaggedTemplateExpression",
];

impl RustLintRule for NoConfusingVoidExpressionRule {
    fn name(&self) -> &'static str {
        "no-confusing-void-expression"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow void-typed expressions in confusing value positions."
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
        if !matches!(
            node.kind.as_str(),
            "AwaitExpression" | "CallExpression" | "TaggedTemplateExpression"
        ) {
            return;
        }
        check_expression(ctx, node);
    }
}

#[derive(Clone, Copy, Debug)]
struct Options {
    ignore_arrow_shorthand: bool,
    ignore_void_operator: bool,
    ignore_void_returning_functions: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            ignore_arrow_shorthand: rule_option_bool(node, "ignoreArrowShorthand").unwrap_or(false),
            ignore_void_operator: rule_option_bool(node, "ignoreVoidOperator").unwrap_or(false),
            ignore_void_returning_functions: rule_option_bool(node, "ignoreVoidReturningFunctions")
                .unwrap_or(false),
        }
    }
}

/// One ancestor descriptor as produced by the host `__ancestorChain` raw fact.
///
/// The list is ordered innermost-first; entry `i` is the parent of the node that
/// `entry[i-1]` describes (with `entry[-1]` describing the void expression's
/// parent). This is a faithful surface for the Go `findInvalidAncestor` walk.
struct Ancestor<'a> {
    raw: &'a Value,
}

impl<'a> Ancestor<'a> {
    fn kind(&self) -> &str {
        self.raw.get("kind").and_then(Value::as_str).unwrap_or("")
    }

    fn operator(&self) -> Option<&str> {
        self.raw.get("operator").and_then(Value::as_str)
    }

    /// Whether the descended-from child sits in the parent's "right operand"
    /// slot (binary/logical) or one of the conditional branch slots.
    fn child_is_right(&self) -> bool {
        self.raw
            .get("childIsRight")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// Whether the descended-from child sits in the parent's "left operand" slot.
    fn child_is_left(&self) -> bool {
        self.raw
            .get("childIsLeft")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// Whether the descended-from child is the consequent or alternate of a
    /// conditional (ternary) expression.
    fn child_is_branch(&self) -> bool {
        self.raw
            .get("childIsBranch")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// For an arrow-function ancestor: whether the host considers it a
    /// void-returning function (raw checker query).
    fn is_void_returning_function(&self) -> bool {
        self.raw
            .get("isVoidReturningFunction")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// For a return-statement ancestor: whether the enclosing function is
    /// void-returning (raw checker query).
    fn enclosing_function_is_void_returning(&self) -> bool {
        self.raw
            .get("enclosingFunctionIsVoidReturning")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// For a return-statement ancestor: whether it is the final statement of the
    /// enclosing function block.
    fn is_final_return(&self) -> bool {
        self.raw
            .get("isFinalReturn")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

fn is_logical_operator(operator: &str) -> bool {
    matches!(operator, "&&" | "||" | "??")
}

/// Faithful port of `findInvalidAncestor`. Returns the index of the invalid
/// ancestor in the chain, or `None` if the void expression is in a valid
/// position.
fn find_invalid_ancestor(chain: &[Ancestor<'_>], opts: Options) -> Option<usize> {
    let mut index = 0;
    while index < chain.len() {
        let parent = &chain[index];
        match parent.kind() {
            // Parens are transparent in TS-ESTree, but the host may still surface
            // them; treat as the Go ParenthesizedExpression `continue`.
            "ParenthesizedExpression" | "TSParenthesizedType" => {
                index += 1;
                continue;
            }
            "LogicalExpression" | "BinaryExpression" => {
                if let Some(operator) = parent.operator() {
                    if is_logical_operator(operator) && parent.child_is_right() {
                        // e.g. `x && console.log(x)` -- valid only if the next
                        // ancestor is valid.
                        index += 1;
                        continue;
                    }
                }
                // Other binary operators (incl. comma handled below) fall through.
            }
            "SequenceExpression"
                // ts-eslint's comma handling: `a, console.log(b)` where the void
                // expression is the LEFT operand is valid.
                if parent.child_is_left() => {
                    return None;
                }
                // Right operand of a sequence falls through (invalid).
            "ExpressionStatement" => {
                // e.g. `{ console.log("foo"); }` -- always valid.
                return None;
            }
            "ConditionalExpression"
                if parent.child_is_branch() => {
                    // e.g. `cond ? console.log(true) : console.log(false)` --
                    // valid only if the next ancestor is valid.
                    index += 1;
                    continue;
                }
            "ArrowFunctionExpression"
                if opts.ignore_arrow_shorthand => {
                    // e.g. `() => console.log("foo")` -- valid with the option.
                    return None;
                }
            "UnaryExpression"
                // The TS-ESTree representation of the `void` operator.
                if parent.operator() == Some("void") && opts.ignore_void_operator => {
                    return None;
                }
            _ => {}
        }
        break;
    }

    if index >= chain.len() {
        return None;
    }
    Some(index)
}

fn check_expression(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let opts = Options::from_node(node);

    // Bail unless the expression's type is void-like.
    if !is_void_like(&node.type_texts) {
        return;
    }

    let chain = ancestor_chain(node);
    let Some(invalid_index) = find_invalid_ancestor(&chain, opts) else {
        return;
    };
    let invalid = &chain[invalid_index];

    let insert_void_fix =
        || LintFix::replace_range(TextRange::new(node.range.start, node.range.start), "void ");

    match invalid.kind() {
        "ArrowFunctionExpression" => {
            if opts.ignore_void_returning_functions && invalid.is_void_returning_function() {
                return;
            }
            if opts.ignore_void_operator {
                // Go emits `void `-insert as an auto-fix here. Token-position-
                // dependent fixes are a documented fix-only gap (see parityNotes);
                // we report the diagnostic with the correct message id only.
                ctx.report("invalidVoidExprArrowWrapVoid", node.range);
                return;
            }
            // invalidVoidExprArrow: braces fix only when the arrow body is the
            // void expression itself (non-block, fixable). We can only safely
            // emit the brace fix when the host marks the arrow body range.
            ctx.report("invalidVoidExprArrow", node.range);
        }
        "ReturnStatement" => {
            if opts.ignore_void_returning_functions
                && invalid.enclosing_function_is_void_returning()
            {
                return;
            }
            if opts.ignore_void_operator {
                ctx.report("invalidVoidExprReturnWrapVoid", node.range);
                return;
            }
            if invalid.is_final_return() {
                ctx.report("invalidVoidExprReturnLast", node.range);
                return;
            }
            ctx.report("invalidVoidExprReturn", node.range);
        }
        _ => {
            if opts.ignore_void_operator {
                ctx.report_with_suggestions(
                    "invalidVoidExprWrapVoid",
                    node.range,
                    vec![LintSuggestion {
                        message_id: "voidExprWrapVoid".to_owned(),
                        message: "Mark with an explicit `void` operator.".to_owned(),
                        fixes: vec![insert_void_fix()],
                    }],
                );
                return;
            }
            ctx.report("invalidVoidExpr", node.range);
        }
    }
}

/// Reads the host-provided `__ancestorChain` raw fact into ancestor descriptors.
fn ancestor_chain(node: &LintNode) -> Vec<Ancestor<'_>> {
    node.fields
        .get("__ancestorChain")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(|raw| Ancestor { raw }).collect())
        .unwrap_or_default()
}

/// Whether any rendered type text is void-like (matches `TypeFlagsVoidLike`:
/// `void` or `undefined`). Mirrors `IsTypeFlagSet(t, VoidLike)` over union parts.
fn is_void_like(type_texts: &[String]) -> bool {
    type_texts.iter().any(|text| {
        let trimmed = text.trim();
        if trimmed == "void" || trimmed == "undefined" {
            return true;
        }
        split_top_level_type_text(trimmed, '|').iter().any(|part| {
            let part = part.trim();
            part == "void"
                || part == "undefined"
                || classify_type_text(Some(part)) == TypeTextKind::Nullish && part != "null"
        })
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::RuleContext;
    use super::super::super::RustLintRule;
    use super::super::super::{LintNode, TextRange};
    use super::NoConfusingVoidExpressionRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoConfusingVoidExpressionRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn void_call(start: u32, end: u32) -> LintNode {
        let mut node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": { "start": start, "end": end },
            "typeTexts": ["void"],
            "fields": {},
        }))
        .unwrap();
        // ensure maps are present
        node.children = BTreeMap::new();
        node
    }

    fn set_chain(node: &mut LintNode, chain: serde_json::Value) {
        node.fields.insert("__ancestorChain".to_owned(), chain);
    }

    fn set_options(node: &mut LintNode, options: serde_json::Value) {
        node.fields
            .insert("__ruleOptions".to_owned(), json!([options]));
    }

    #[test]
    fn reports_void_in_variable_initializer() {
        // const x = console.log('foo');  -> VariableDeclarator parent => invalid.
        let mut node = void_call(10, 28);
        set_chain(
            &mut node,
            json!([{ "kind": "VariableDeclarator", "start": 0, "end": 28 }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "invalidVoidExpr");
        assert_eq!(diagnostics[0].range, TextRange::new(10, 28));
    }

    #[test]
    fn reports_void_arrow_shorthand() {
        // () => console.log('foo');  -> ArrowFunctionExpression parent.
        let mut node = void_call(6, 24);
        set_chain(
            &mut node,
            json!([{ "kind": "ArrowFunctionExpression", "start": 0, "end": 24, "arrowBodyIsBlock": false }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "invalidVoidExprArrow");
    }

    #[test]
    fn reports_final_return() {
        // function f() { return console.log('bar'); }
        let mut node = void_call(20, 38);
        set_chain(
            &mut node,
            json!([{ "kind": "ReturnStatement", "start": 13, "end": 39, "isFinalReturn": true }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "invalidVoidExprReturnLast");
    }

    #[test]
    fn reports_non_final_return() {
        let mut node = void_call(20, 38);
        set_chain(
            &mut node,
            json!([{ "kind": "ReturnStatement", "start": 13, "end": 39, "isFinalReturn": false }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "invalidVoidExprReturn");
    }

    #[test]
    fn ignores_expression_statement() {
        // console.log('foo');  -> ExpressionStatement parent => valid.
        let mut node = void_call(0, 18);
        set_chain(
            &mut node,
            json!([{ "kind": "ExpressionStatement", "start": 0, "end": 19 }]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_logical_right_operand_with_valid_outer() {
        // foo && console.log(foo);  -> LogicalExpression(right) then ExpressionStatement.
        let mut node = void_call(7, 25);
        set_chain(
            &mut node,
            json!([
                { "kind": "LogicalExpression", "operator": "&&", "childIsRight": true, "start": 0, "end": 25 },
                { "kind": "ExpressionStatement", "start": 0, "end": 26 }
            ]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_arrow_shorthand_with_option() {
        let mut node = void_call(6, 24);
        set_options(&mut node, json!({ "ignoreArrowShorthand": true }));
        set_chain(
            &mut node,
            json!([{ "kind": "ArrowFunctionExpression", "start": 0, "end": 24 }]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn wraps_void_with_option_emits_suggestion() {
        // console.error(console.log('foo')); with ignoreVoidOperator
        let mut node = void_call(14, 32);
        set_options(&mut node, json!({ "ignoreVoidOperator": true }));
        set_chain(
            &mut node,
            json!([{ "kind": "CallExpression", "childIsRight": false, "start": 0, "end": 34 }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "invalidVoidExprWrapVoid");
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(diagnostics[0].suggestions[0].message_id, "voidExprWrapVoid");
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].replacement_text,
            "void "
        );
    }

    #[test]
    fn ignores_non_void_type() {
        let mut node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": { "start": 10, "end": 28 },
            "typeTexts": ["string"],
        }))
        .unwrap();
        set_chain(
            &mut node,
            json!([{ "kind": "VariableDeclarator", "start": 0, "end": 28 }]),
        );
        assert!(run(&node).is_empty());
    }
}

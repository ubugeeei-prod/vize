use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{child_list, is_any_like_node, strip_chain_expression};
use crate::utils::{is_any_like_type_texts, is_array_like_type_texts, is_unknown_like_type_texts};

/// Type-aware rule that disallows calling functions with `any`-typed arguments
/// where the matching parameter expects a more specific type.
///
/// Faithful port of the tsgolint `no-unsafe-argument` rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeArgumentRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unsafeArgument",
        description: "Unsafe argument of an any typed value assigned to a non-any parameter.",
    },
    RuleMessage {
        id: "unsafeArraySpread",
        description: "Unsafe spread of an any array type.",
    },
    RuleMessage {
        id: "unsafeSpread",
        description: "Unsafe spread of an any type.",
    },
    RuleMessage {
        id: "unsafeTupleSpread",
        description: "Unsafe spread of a tuple type with an any element assigned to a non-any parameter.",
    },
];

const LISTENERS: &[&str] = &[
    "CallExpression",
    "NewExpression",
    "TaggedTemplateExpression",
];

impl RustLintRule for NoUnsafeArgumentRule {
    fn name(&self) -> &'static str {
        "no-unsafe-argument"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow calling a function with a value of type `any`."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "CallExpression" | "NewExpression" => {
                let arguments = collect_call_arguments(node);
                check_unsafe_arguments(ctx, node, &arguments, /* is_tagged_template */ false);
            }
            "TaggedTemplateExpression" => {
                let arguments = collect_tagged_template_arguments(node);
                check_unsafe_arguments(ctx, node, &arguments, /* is_tagged_template */ true);
            }
            _ => {}
        }
    }
}

/// Returns the argument nodes of a `CallExpression`/`NewExpression`.
fn collect_call_arguments(node: &LintNode) -> Vec<&LintNode> {
    child_list(node, "arguments").iter().collect()
}

/// Returns the embedded expression nodes of a tagged template literal.
///
/// In the Go rule the first parameter (the `TemplateStringsArray`) is consumed
/// separately; here we mirror that by skipping the first expected parameter when
/// matching template spans against parameter types.
fn collect_tagged_template_arguments(node: &LintNode) -> Vec<&LintNode> {
    let Some(template) = node.child("quasi").or_else(|| node.child("template")) else {
        return Vec::new();
    };
    child_list(template, "expressions").iter().collect()
}

/// Reads the host-provided per-argument expected parameter type texts.
///
/// `__expectedArgumentTypeTexts[i]` is the raw rendered type-text list of the
/// parameter that argument `i` is assigned to (the host already resolves the
/// call signature and clamps trailing rest arguments to the rest element type).
fn expected_argument_type_texts(node: &LintNode) -> Vec<Vec<String>> {
    node.fields
        .get("__expectedArgumentTypeTexts")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_array()
                        .map(|texts| {
                            texts
                                .iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Reads the host-provided rendered element type texts of a spread tuple
/// argument, in declaration order. Empty when the argument is not a tuple.
fn spread_tuple_element_type_texts(spread: &LintNode) -> Vec<Vec<String>> {
    spread
        .fields
        .get("__spreadTupleElementTypeTexts")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_array()
                        .map(|texts| {
                            texts
                                .iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Returns true when the host marked the spread tuple as ending in a rest
/// element (so remaining parameters should be treated as consumed).
fn spread_tuple_has_rest(spread: &LintNode) -> bool {
    spread.field_bool("__spreadTupleHasRest").unwrap_or(false)
}

/// Returns true when the call's callee resolves to an `any` type. Such calls are
/// covered by `no-unsafe-call`, so this rule ignores them (mirrors the Go rule's
/// `IsTypeAnyType(GetTypeAtLocation(callee))` guard).
fn callee_is_any(node: &LintNode) -> bool {
    if node.field_bool("__calleeIsAnyType").unwrap_or(false) {
        return true;
    }
    let callee = node
        .child("callee")
        .or_else(|| node.child("tag"))
        .or_else(|| node.child("expression"));
    callee
        .map(strip_chain_expression)
        .is_some_and(is_any_like_node)
}

/// Returns true when the parameter type texts represent `any` or `unknown`.
///
/// Mirrors `IsUnsafeAssignment`: assigning `any` to `unknown` (or to `any`) is
/// safe, everything else with an `any` sender is unsafe.
fn parameter_is_any_or_unknown(expected: &[String]) -> bool {
    expected.is_empty() || is_any_like_type_texts(expected) || is_unknown_like_type_texts(expected)
}

/// Returns true when the type texts represent the intrinsic `error` type that
/// TypeScript synthesizes for unresolved symbols. The Go rule treats these as
/// unsafe (`IsIntrinsicErrorType`).
fn is_error_typed(type_texts: &[String]) -> bool {
    type_texts.iter().any(|text| text.trim() == "error")
}

/// True when the argument should be treated as an unsafe `any`/error sender.
fn is_unsafe_sender(node: &LintNode) -> bool {
    is_any_like_node(node) || is_error_typed(&node.type_texts)
}

fn check_unsafe_arguments(
    ctx: &mut RuleContext<'_>,
    node: &LintNode,
    arguments: &[&LintNode],
    is_tagged_template: bool,
) {
    if arguments.is_empty() {
        return;
    }

    // Ignore any-typed calls; these are caught by `no-unsafe-call`.
    if callee_is_any(node) {
        return;
    }

    let expected_types = expected_argument_type_texts(node);

    // For tagged templates the first parameter is the `TemplateStringsArray`,
    // which the template spans never match against. The host emits expected
    // types positionally for the template spans, so no offset is needed when the
    // host already excludes the strings array; when it does not we skip index 0.
    let parameter_offset =
        usize::from(is_tagged_template && expected_types.len() > arguments.len());

    for (index, argument) in arguments.iter().enumerate() {
        let argument = *argument;
        if argument.kind == "SpreadElement" {
            check_spread_argument(ctx, argument, &expected_types, index + parameter_offset);
            continue;
        }

        let expected = expected_types
            .get(index + parameter_offset)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        // No matching parameter (e.g. excess argument with no rest) -> skip.
        if expected.is_empty() {
            continue;
        }
        if !is_unsafe_sender(argument) {
            continue;
        }
        if parameter_is_any_or_unknown(expected) {
            continue;
        }
        ctx.report("unsafeArgument", argument.range);
    }
}

fn check_spread_argument(
    ctx: &mut RuleContext<'_>,
    spread: &LintNode,
    expected_types: &[Vec<String>],
    base_index: usize,
) {
    let Some(inner) = spread.child("argument").map(strip_chain_expression) else {
        return;
    };

    // foo(...any)
    if is_any_like_node(inner) {
        ctx.report("unsafeSpread", spread.range);
        return;
    }

    // foo(...[tuple1, tuple2])
    //
    // The tuple branch must run BEFORE the array branch: a tuple type is
    // rendered with array-like brackets (e.g. `[string, any, number]`) and
    // would otherwise be misclassified by `is_array_like_type_texts`. The host
    // marks genuine tuple arguments with `__spreadTupleElementTypeTexts`, which
    // mirrors the Go rule's `checker.IsTupleType` branch that takes precedence
    // over `IsTypeAnyArrayType`.
    let tuple_elements = spread_tuple_element_type_texts(spread);
    if !tuple_elements.is_empty() {
        for (offset, element) in tuple_elements.iter().enumerate() {
            let expected = expected_types
                .get(base_index + offset)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if expected.is_empty() {
                continue;
            }
            let element_unsafe = is_any_like_type_texts(element) || is_error_typed(element);
            if element_unsafe && !parameter_is_any_or_unknown(expected) {
                ctx.report("unsafeTupleSpread", spread.range);
            }
        }
        // `__spreadTupleHasRest` (consumeRemainingArguments) is read by the host
        // facts but the remaining-argument bookkeeping is simplified, matching
        // Go's documented "rare and complicated, ignored" caveat.
        let _ = spread_tuple_has_rest(spread);
        return;
    }

    // foo(...any[]) or foo(...error[])
    //
    // Only genuine array types reach here (tuples were handled above), so a
    // bracketed tuple text can no longer be misread as an `any[]` spread.
    if is_array_like_type_texts(&inner.type_texts) && array_element_is_any_or_error(inner) {
        ctx.report("unsafeArraySpread", spread.range);
    }

    // Anything else iterable is intentionally ignored, mirroring the Go rule.
}

/// Returns true when the spread argument is an array whose element type is `any`
/// or the intrinsic `error` type.
fn array_element_is_any_or_error(inner: &LintNode) -> bool {
    if let Some(element_types) = inner
        .fields
        .get("__arrayElementTypeTexts")
        .and_then(Value::as_array)
    {
        let texts: Vec<String> = element_types
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        return is_any_like_type_texts(&texts) || is_error_typed(&texts);
    }
    // Fallback: inspect the rendered array type text directly.
    inner.type_texts.iter().any(|text| {
        let text = text.trim();
        text == "any[]"
            || text == "readonly any[]"
            || text == "error[]"
            || text == "readonly error[]"
            || text.starts_with("Array<any>")
            || text.starts_with("ReadonlyArray<any>")
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnsafeArgumentRule;

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

    fn any_arg(range: TextRange) -> LintNode {
        let mut arg = node("TSAsExpression", range);
        arg.fields
            .insert("__typeAnnotationText".to_owned(), json!("any"));
        arg.type_texts = vec!["any".to_owned()];
        // typeAnnotation child so `is_any_like_node` matches the `as any` form.
        let mut annotation = node("TSAnyKeyword", range);
        annotation.type_texts = vec!["any".to_owned()];
        arg.children.insert("typeAnnotation".to_owned(), annotation);
        arg
    }

    fn typed_arg(kind: &str, range: TextRange, type_text: &str) -> LintNode {
        let mut arg = node(kind, range);
        arg.type_texts = vec![type_text.to_owned()];
        arg
    }

    fn call(arguments: Vec<LintNode>, expected: serde_json::Value) -> LintNode {
        let mut callee = node("Identifier", TextRange::new(0, 3));
        callee.fields.insert("name".to_owned(), json!("foo"));
        callee.type_texts = vec!["(arg: number) => void".to_owned()];
        let mut call = node("CallExpression", TextRange::new(0, 20));
        call.children.insert("callee".to_owned(), callee);
        call.child_lists.insert("arguments".to_owned(), arguments);
        call.fields
            .insert("__expectedArgumentTypeTexts".to_owned(), expected);
        call
    }

    fn run(node: &LintNode) -> Vec<String> {
        let rule = NoUnsafeArgumentRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    #[test]
    fn reports_any_argument_to_typed_parameter() {
        // foo(1 as any) where foo(arg: number)
        let call = call(vec![any_arg(TextRange::new(4, 12))], json!([["number"]]));
        assert_eq!(run(&call), vec!["unsafeArgument".to_owned()]);
    }

    #[test]
    fn reports_any_in_second_position() {
        // foo(1, 1 as any) where foo(arg1: number, arg2: string)
        let call = call(
            vec![
                typed_arg("Literal", TextRange::new(4, 5), "number"),
                any_arg(TextRange::new(7, 15)),
            ],
            json!([["number"], ["string"]]),
        );
        assert_eq!(run(&call), vec!["unsafeArgument".to_owned()]);
    }

    #[test]
    fn does_not_report_any_to_any_parameter() {
        // foo(1 as any) where foo(arg: any)
        let call = call(vec![any_arg(TextRange::new(4, 12))], json!([["any"]]));
        assert!(run(&call).is_empty());
    }

    #[test]
    fn does_not_report_any_to_unknown_parameter() {
        // foo(1 as any) where foo(arg: unknown)
        let call = call(vec![any_arg(TextRange::new(4, 12))], json!([["unknown"]]));
        assert!(run(&call).is_empty());
    }

    #[test]
    fn does_not_report_safe_typed_argument() {
        // foo(1, 'a') where foo(arg1: number, arg2: string)
        let call = call(
            vec![
                typed_arg("Literal", TextRange::new(4, 5), "number"),
                typed_arg("Literal", TextRange::new(7, 10), "string"),
            ],
            json!([["number"], ["string"]]),
        );
        assert!(run(&call).is_empty());
    }

    #[test]
    fn ignores_any_typed_callee() {
        // (foo as any)(1 as any) -> callee any, covered by no-unsafe-call.
        let mut callee = node("Identifier", TextRange::new(0, 3));
        callee.type_texts = vec!["any".to_owned()];
        let mut call = node("CallExpression", TextRange::new(0, 20));
        call.children.insert("callee".to_owned(), callee);
        call.child_lists
            .insert("arguments".to_owned(), vec![any_arg(TextRange::new(4, 12))]);
        call.fields.insert(
            "__expectedArgumentTypeTexts".to_owned(),
            json!([["number"]]),
        );
        assert!(run(&call).is_empty());
    }

    #[test]
    fn reports_spread_of_any() {
        // foo(...(x as any))
        let mut inner = node("TSAsExpression", TextRange::new(8, 17));
        inner.type_texts = vec!["any".to_owned()];
        inner
            .fields
            .insert("__typeAnnotationText".to_owned(), json!("any"));
        let mut annotation = node("TSAnyKeyword", TextRange::new(13, 16));
        annotation.type_texts = vec!["any".to_owned()];
        inner
            .children
            .insert("typeAnnotation".to_owned(), annotation);

        let mut spread = node("SpreadElement", TextRange::new(4, 18));
        spread.children.insert("argument".to_owned(), inner);

        let call = call(vec![spread], json!([["string"], ["number"]]));
        assert_eq!(run(&call), vec!["unsafeSpread".to_owned()]);
    }

    #[test]
    fn reports_spread_of_any_array() {
        // foo(...(x as any[]))
        let mut inner = node("TSAsExpression", TextRange::new(8, 19));
        inner.type_texts = vec!["any[]".to_owned()];

        let mut spread = node("SpreadElement", TextRange::new(4, 20));
        spread.children.insert("argument".to_owned(), inner);

        let call = call(vec![spread], json!([["string"], ["number"]]));
        assert_eq!(run(&call), vec!["unsafeArraySpread".to_owned()]);
    }

    #[test]
    fn reports_unsafe_tuple_spread() {
        // const x = ['a', 1 as any] as const; foo(...x)
        let mut spread = node("SpreadElement", TextRange::new(4, 8));
        let mut inner = node("Identifier", TextRange::new(7, 8));
        inner.type_texts = vec!["readonly [\"a\", any]".to_owned()];
        spread.children.insert("argument".to_owned(), inner);
        spread.fields.insert(
            "__spreadTupleElementTypeTexts".to_owned(),
            json!([["\"a\""], ["any"]]),
        );

        let call = call(vec![spread], json!([["string"], ["number"]]));
        assert_eq!(run(&call), vec!["unsafeTupleSpread".to_owned()]);
    }

    #[test]
    fn reports_unsafe_inline_tuple_spread_not_array() {
        // foo(...(['foo', 1, 2] as [string, any, number]))
        // The tuple text is bracketed (`[string, any, number]`) and would be
        // misread as an array spread if the array branch ran first.
        let mut spread = node("SpreadElement", TextRange::new(4, 48));
        let mut inner = node("TSAsExpression", TextRange::new(8, 47));
        inner.type_texts = vec!["[string, any, number]".to_owned()];
        spread.children.insert("argument".to_owned(), inner);
        spread.fields.insert(
            "__spreadTupleElementTypeTexts".to_owned(),
            json!([["string"], ["any"], ["number"]]),
        );

        let call = call(vec![spread], json!([["string"], ["number"]]));
        assert_eq!(run(&call), vec!["unsafeTupleSpread".to_owned()]);
    }

    #[test]
    fn does_not_report_safe_tuple_spread() {
        // const x = [1, 2] as const; foo(...x) where foo(arg: number, arg2: number)
        let mut spread = node("SpreadElement", TextRange::new(4, 8));
        let mut inner = node("Identifier", TextRange::new(7, 8));
        inner.type_texts = vec!["readonly [1, 2]".to_owned()];
        spread.children.insert("argument".to_owned(), inner);
        spread.fields.insert(
            "__spreadTupleElementTypeTexts".to_owned(),
            json!([["1"], ["2"]]),
        );

        let call = call(vec![spread], json!([["number"], ["number"]]));
        assert!(run(&call).is_empty());
    }
}

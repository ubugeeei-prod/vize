use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::is_identifier_named;
use crate::utils::{is_any_like_type_texts, is_unknown_like_type_texts, split_type_text};

/// Type-aware rule that flags default assignments which can never be used
/// because the assignment target can never be nullish.
///
/// This is a faithful port of the tsgolint Go rule
/// `no-useless-default-assignment`. The Go rule listens on `Parameter` and
/// `BindingElement` nodes; in the TS-ESTree shape those default-value
/// constructs are represented by an `AssignmentPattern` node whose `left` is the
/// binding target (identifier / object pattern / array pattern) and whose
/// `right` is the default value. The parent kind disambiguates a function
/// parameter from a destructuring property/element.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUselessDefaultAssignmentRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "noStrictNullCheck",
        description: "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    },
    RuleMessage {
        id: "preferOptionalSyntax",
        description: "Using `= undefined` to make a parameter optional adds unnecessary runtime logic. Use the `?` optional syntax instead.",
    },
    RuleMessage {
        id: "uselessDefaultAssignment",
        description: "Default value is useless because the value is not nullish. This default assignment will never be used.",
    },
    RuleMessage {
        id: "uselessUndefined",
        description: "Default value is useless because it is undefined. Optional values are already undefined by default.",
    },
];

const LISTENERS: &[&str] = &["AssignmentPattern"];

impl RustLintRule for NoUselessDefaultAssignmentRule {
    fn name(&self) -> &'static str {
        "no-useless-default-assignment"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow default value assignments that can never be used."
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
        if node.kind != "AssignmentPattern" {
            return;
        }

        // The Go rule first reports `noStrictNullCheck` once at range (0, 0)
        // when strict null checks are disabled. The host attaches the raw
        // compiler-option fact `__strictNullChecks` to every emitted node; the
        // host is responsible for emitting this once globally, but mirroring the
        // upstream we surface it here when the fact is explicitly `false`.
        if node.field_bool("__strictNullChecks") == Some(false)
            && node.field_bool("__strictNullCheckReported") != Some(true)
        {
            ctx.report("noStrictNullCheck", TextRange::new(0, 0));
        }

        let Some(left) = node.child("left") else {
            return;
        };
        let Some(right) = node.child("right") else {
            return;
        };

        let assignment_type = assignment_type(node);

        // `= undefined` is always useless (or, for parameters with a
        // nullable type annotation, should prefer the `?` optional syntax).
        if is_identifier_named(right, "undefined") {
            if assignment_type == AssignmentType::Parameter {
                if annotation_can_be_undefined(node, left) {
                    report_prefer_optional_syntax(ctx, node, left);
                    return;
                }
                report_remove_default(ctx, "uselessUndefined", node, left);
                return;
            }
            report_remove_default(ctx, "uselessUndefined", node, left);
            return;
        }

        // For non-undefined defaults the default is useless only when the
        // target can never be `undefined`. The host computes the resolved
        // target type via the checker (`__targetTypeTexts`) and whether the
        // property/parameter is optional (`__targetOptional`). When the target
        // is optional the default is legitimately reachable, so we bail.
        if node.field_bool("__targetOptional") == Some(true) {
            return;
        }

        // If the host could not resolve a target type (unknown source, index
        // signatures, `noUncheckedIndexedAccess`, generic functions, etc.) we
        // conservatively do not report, matching the Go rule's many `return nil`
        // bail-outs.
        let Some(target_type_texts) = target_type_texts(node) else {
            return;
        };

        if can_be_undefined(&target_type_texts) {
            return;
        }

        report_remove_default(ctx, "uselessDefaultAssignment", node, left);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AssignmentType {
    Parameter,
    Property,
}

/// Mirrors the Go rule's `assignmentType` discrimination. In TS-ESTree a
/// default-valued function parameter has the function node as the
/// `AssignmentPattern` parent; a destructured property/element has a `Property`
/// or `ArrayPattern` parent.
fn assignment_type(node: &LintNode) -> AssignmentType {
    match node.field_str("__parentKind") {
        Some(
            "FunctionDeclaration"
            | "FunctionExpression"
            | "ArrowFunctionExpression"
            | "TSDeclareFunction"
            | "TSEmptyBodyFunctionExpression",
        ) => AssignmentType::Parameter,
        _ => AssignmentType::Property,
    }
}

/// Resolved type texts of the assignment target, as computed by the host
/// checker. Equivalent to the Go rule's `getTypeOfBindingElement` /
/// parameter-symbol type lookup.
fn target_type_texts(node: &LintNode) -> Option<Vec<String>> {
    node.fields
        .get("__targetTypeTexts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .filter(|texts| !texts.is_empty())
}

/// Faithful port of the Go `canBeUndefined`: `any`/`unknown` types can be
/// undefined, and any union part that is `undefined` counts.
fn can_be_undefined(type_texts: &[String]) -> bool {
    if is_any_like_type_texts(type_texts) || is_unknown_like_type_texts(type_texts) {
        return true;
    }
    type_texts.iter().any(|text| {
        split_type_text(text)
            .iter()
            .any(|part| part.trim() == "undefined")
    })
}

/// Whether the parameter's explicit type annotation can already be `undefined`,
/// driving the `preferOptionalSyntax` branch. Derived from the host-provided
/// annotation text on the binding target when present, else from the raw fact.
fn annotation_can_be_undefined(node: &LintNode, left: &LintNode) -> bool {
    if node.field_bool("__annotationCanBeUndefined") == Some(true) {
        return true;
    }
    let Some(annotation) = left
        .field_str("__typeAnnotationText")
        .or_else(|| node.field_str("__typeAnnotationText"))
    else {
        return false;
    };
    let texts = [annotation.to_owned()];
    can_be_undefined(&texts)
}

/// Computes the byte range of the default-assignment tail (`= default`) that the
/// remove fix deletes. Mirrors Go `getDefaultAssignmentStart`: it begins at the
/// end of the binding target (after its optional type annotation) and runs to
/// the end of the whole `AssignmentPattern` node.
fn default_assignment_range(node: &LintNode, left: &LintNode) -> TextRange {
    TextRange::new(left.range.end, node.range.end)
}

fn report_remove_default(
    ctx: &mut RuleContext<'_>,
    message_id: &'static str,
    node: &LintNode,
    left: &LintNode,
) {
    let right_range = node
        .child("right")
        .map(|right| right.range)
        .unwrap_or(node.range);
    let fix = LintFix::remove_range(default_assignment_range(node, left));
    ctx.report_with_suggestions(
        message_id,
        right_range,
        vec![LintSuggestion {
            message_id: message_id.to_owned(),
            message: "Remove the default assignment".to_owned(),
            fixes: vec![fix],
        }],
    );
}

/// End byte offset of the parameter's *name token* (the trimmed identifier),
/// where Go inserts `?` for the `preferOptionalSyntax` fix
/// (`TrimNodeTextRange(name).End()`).
///
/// NOTE: in TS-ESTree the `AssignmentPattern.left` `Identifier` node's `range`
/// extends past the name to cover the attached `typeAnnotation`, so
/// `left.range.end` is NOT the name end whenever the parameter is type
/// annotated — and `preferOptionalSyntax` only fires for annotated parameters.
/// We therefore consume the host fact `__nameEnd` (the trimmed name-token end);
/// if absent we fall back to the `typeAnnotation` child's start (correct unless
/// there are comments between the name and `:`) and finally to `left.range.end`.
fn name_end(node: &LintNode, left: &LintNode) -> u32 {
    if let Some(end) = node
        .field_f64("__nameEnd")
        .or_else(|| left.field_f64("__nameEnd"))
    {
        return end as u32;
    }
    if let Some(annotation) = left.child("typeAnnotation") {
        return annotation.range.start;
    }
    left.range.end
}

fn report_prefer_optional_syntax(ctx: &mut RuleContext<'_>, node: &LintNode, left: &LintNode) {
    let right_range = node
        .child("right")
        .map(|right| right.range)
        .unwrap_or(node.range);
    let mut fixes = vec![LintFix::remove_range(default_assignment_range(node, left))];
    // For an identifier parameter, also append `?` after the *name token* to
    // make it optional (Go `reportPreferOptionalSyntax`). The insertion must be
    // right after the name, NOT after the (annotation-spanning) `left` range.
    if left.kind == "Identifier" {
        let name_end = name_end(node, left);
        let insertion = TextRange::new(name_end, name_end);
        fixes.push(LintFix::replace_range(insertion, "?"));
    }
    ctx.report_with_suggestions(
        "preferOptionalSyntax",
        right_range,
        vec![LintSuggestion {
            message_id: "preferOptionalSyntax".to_owned(),
            message: "Remove the default assignment".to_owned(),
            fixes,
        }],
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext};
    use super::NoUselessDefaultAssignmentRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoUselessDefaultAssignmentRule;
        let mut ctx = RuleContext::new(&rule);
        <NoUselessDefaultAssignmentRule as super::RustLintRule>::check(&rule, &mut ctx, node);
        ctx.finish()
    }

    fn assignment_pattern(
        parent_kind: &str,
        right: serde_json::Value,
        target_type_texts: serde_json::Value,
        extra_fields: serde_json::Value,
    ) -> LintNode {
        let mut fields = json!({ "__parentKind": parent_kind });
        if !target_type_texts.is_null() {
            fields["__targetTypeTexts"] = target_type_texts;
        }
        if let Some(extra) = extra_fields.as_object() {
            for (key, value) in extra {
                fields[key] = value.clone();
            }
        }
        serde_json::from_value(json!({
            "kind": "AssignmentPattern",
            "range": { "start": 10, "end": 20 },
            "fields": fields,
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 10, "end": 13 },
                    "fields": { "name": "foo" }
                },
                "right": right
            }
        }))
        .unwrap()
    }

    // ---- should report ----

    #[test]
    fn reports_useless_default_on_non_nullish_property() {
        // function Bar({ foo = '' }: { foo: string }) {}
        let node = assignment_pattern(
            "Property",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "" } }),
            json!(["string"]),
            json!({}),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "uselessDefaultAssignment");
        // Removal fix deletes from end of `left` to end of node.
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].range,
            super::super::super::TextRange::new(13, 20)
        );
    }

    #[test]
    fn reports_useless_undefined_default() {
        // function foo(a = undefined) {}
        let node = assignment_pattern(
            "FunctionDeclaration",
            json!({ "kind": "Identifier", "range": { "start": 16, "end": 25 }, "fields": { "name": "undefined" } }),
            json!(null),
            json!({}),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "uselessUndefined");
    }

    #[test]
    fn reports_prefer_optional_syntax_for_nullable_param_with_undefined() {
        // function f(p2: number | undefined = undefined) {}
        let node = assignment_pattern(
            "FunctionDeclaration",
            json!({ "kind": "Identifier", "range": { "start": 16, "end": 25 }, "fields": { "name": "undefined" } }),
            json!(null),
            json!({ "__annotationCanBeUndefined": true }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferOptionalSyntax");
        // Two fixes: remove default + append `?`.
        assert_eq!(diagnostics[0].suggestions[0].fixes.len(), 2);
        assert_eq!(diagnostics[0].suggestions[0].fixes[1].replacement_text, "?");
    }

    #[test]
    fn prefer_optional_inserts_question_mark_after_name_not_after_annotation() {
        // function f(p2: number | undefined = undefined) {}
        // The `left` Identifier range [16, 34] spans `p2: number | undefined`;
        // the `?` must be inserted at the name-token end (`__nameEnd` = 18),
        // NOT at left.range.end (34), so the output is `p2?: number | undefined`.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "AssignmentPattern",
            "range": { "start": 16, "end": 47 },
            "fields": { "__parentKind": "FunctionDeclaration", "__annotationCanBeUndefined": true },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 16, "end": 34 },
                    "fields": { "name": "p2", "__nameEnd": 18 },
                    "children": {
                        "typeAnnotation": {
                            "kind": "TSTypeAnnotation",
                            "range": { "start": 18, "end": 34 }
                        }
                    }
                },
                "right": { "kind": "Identifier", "range": { "start": 37, "end": 46 }, "fields": { "name": "undefined" } }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "preferOptionalSyntax");
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes.len(), 2);
        // Remove fix: from end of annotation (left.range.end = 34) to node end.
        assert_eq!(fixes[0].range, super::super::super::TextRange::new(34, 47));
        // `?` insertion at the name-token end, not after the annotation.
        assert_eq!(fixes[1].replacement_text, "?");
        assert_eq!(fixes[1].range, super::super::super::TextRange::new(18, 18));
    }

    #[test]
    fn prefer_optional_falls_back_to_annotation_start_without_name_end_fact() {
        // Same as above but no __nameEnd fact: falls back to typeAnnotation start.
        let node: LintNode = serde_json::from_value(json!({
            "kind": "AssignmentPattern",
            "range": { "start": 16, "end": 47 },
            "fields": { "__parentKind": "FunctionDeclaration", "__annotationCanBeUndefined": true },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 16, "end": 34 },
                    "fields": { "name": "p2" },
                    "children": {
                        "typeAnnotation": {
                            "kind": "TSTypeAnnotation",
                            "range": { "start": 18, "end": 34 }
                        }
                    }
                },
                "right": { "kind": "Identifier", "range": { "start": 37, "end": 46 }, "fields": { "name": "undefined" } }
            }
        }))
        .unwrap();
        let diagnostics = run(&node);
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes[1].range, super::super::super::TextRange::new(18, 18));
    }

    // ---- should NOT report ----

    #[test]
    fn ignores_optional_property_default() {
        // function Bar({ foo = '' }: { foo?: string }) {}
        let node = assignment_pattern(
            "Property",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "" } }),
            json!(["string"]),
            json!({ "__targetOptional": true }),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_nullable_target_type() {
        // function test(a: string | undefined = 'default') {}
        let node = assignment_pattern(
            "FunctionDeclaration",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "default" } }),
            json!(["string | undefined"]),
            json!({}),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_any_target_type() {
        // function test(a: any = 'default') {}
        let node = assignment_pattern(
            "FunctionDeclaration",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "default" } }),
            json!(["any"]),
            json!({}),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_unresolved_target_type() {
        // const { value = 'default' } = someUnknownFunction();
        let node = assignment_pattern(
            "Property",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "default" } }),
            json!(null),
            json!({}),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_no_strict_null_check_once() {
        let node = assignment_pattern(
            "FunctionDeclaration",
            json!({ "kind": "Literal", "range": { "start": 16, "end": 18 }, "fields": { "value": "" } }),
            json!(["string"]),
            json!({ "__strictNullChecks": false }),
        );
        let diagnostics = run(&node);
        // noStrictNullCheck + uselessDefaultAssignment
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message_id, "noStrictNullCheck");
        assert_eq!(
            diagnostics[0].range,
            super::super::super::TextRange::new(0, 0)
        );
    }
}

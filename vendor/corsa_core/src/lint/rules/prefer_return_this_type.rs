use super::super::{LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::is_identifier_named;

/// Type-aware rule that prefers `this` as the return type for fluent instance
/// methods of a class that return the receiver.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferReturnThisTypeRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "useThisType",
    description: "Use `this` type instead.",
}];

// Upstream listens on `PropertyDeclaration` (class field whose initializer is a
// function/arrow) and `MethodDeclaration`, and in both cases operates on the
// *function* node (the method itself or the property initializer).
//
// On the host side, the `__nearestClassName` fact (and the type-checker facts
// this rule consumes) are attached to the function-like node directly, by
// walking up to the nearest enclosing class. That is true regardless of whether
// the function is a method `value`, a `PropertyDefinition` initializer, or an
// `AccessorProperty` initializer (`accessor f = ...`), because the host's
// `nearestClassAncestor` skips through those member wrappers.
//
// Therefore we listen ONLY on the function-like nodes. Listening additionally
// on `MethodDefinition`/`PropertyDefinition` would double-report every class
// method, since the member node and its inner function node are visited
// separately and both would resolve to the same function and report.
const LISTENERS: &[&str] = &["FunctionExpression", "ArrowFunctionExpression"];

impl RustLintRule for PreferReturnThisTypeRule {
    fn name(&self) -> &'static str {
        "prefer-return-this-type"
    }

    fn docs_description(&self) -> &'static str {
        "Enforce that `this` is used when only `this` type is returned."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        // Upstream emits an auto-fix (RuleFixReplace). We surface it as a
        // suggestion attached to the diagnostic.
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        check_function(ctx, node);
    }
}

fn check_function(ctx: &mut RuleContext<'_>, function: &LintNode) {
    // Upstream: `returnType := fn.Type(); body := fn.Body()`; bail when either
    // is missing.
    let Some(return_type) = function.child("returnType") else {
        return;
    };
    let Some(body) = function.child("body") else {
        return;
    };

    // Upstream: `className := originalClass.Name()`; bail when the class is
    // anonymous. The host supplies the nearest class name as a raw fact, set
    // only when the function is enclosed by a *named* class.
    let Some(class_name) = function.field_str("__nearestClassName") else {
        return;
    };

    // Upstream: `node := tryGetNameInTypeNode(className.Text(), returnType)`;
    // the matched type-reference node is both the report target and fix target.
    let Some(type_node) = try_get_name_in_type_node(class_name, return_type) else {
        return;
    };

    // Upstream: skip when the first parameter is an explicit `this` parameter.
    if first_param_is_this(function) {
        return;
    }

    let body_is_block = body.kind == "BlockStatement";

    if body_is_block {
        // Upstream block-body logic walks every return statement:
        //   - a `this` return sets hasReturnThis
        //   - a return whose type is exactly the class type sets
        //     hasReturnClassType (and aborts the rest)
        // Report only when there is at least one `this` return and no class-type
        // return.
        let has_return_class_type = function
            .field_bool("__returnsContainClassType")
            .unwrap_or(false);
        let has_return_this = function
            .field_bool("__returnsContainThisType")
            .unwrap_or(false)
            || any_return_is_this_expression(body);

        if has_return_class_type || !has_return_this {
            return;
        }
    } else {
        // Upstream arrow/concise-body logic: report only when the body's type is
        // the class's `this` type.
        let body_is_this_expression = body.kind == "ThisExpression";
        let body_is_this_type = function
            .field_bool("__conciseBodyIsThisType")
            .unwrap_or(false)
            || body_is_this_expression;
        if !body_is_this_type {
            return;
        }
    }

    let suggestion = LintSuggestion {
        message_id: "useThisType".to_owned(),
        message: "Use `this` type instead.".to_owned(),
        fixes: vec![LintFix::replace_range(type_node.range, "this")],
    };
    ctx.report_with_suggestions("useThisType", type_node.range, vec![suggestion]);
}

/// Mirrors upstream `tryGetNameInTypeNode`: walks through parenthesized and
/// union types looking for a type reference whose name matches `name`.
fn try_get_name_in_type_node<'a>(name: &str, node: &'a LintNode) -> Option<&'a LintNode> {
    let node = skip_type_annotation(node);
    match node.kind.as_str() {
        "TSParenthesizedType" => node
            .child("typeAnnotation")
            .and_then(|inner| try_get_name_in_type_node(name, inner)),
        "TSTypeReference" => {
            let type_name = node.child("typeName")?;
            (type_name.kind == "Identifier" && type_name.field_str("name") == Some(name))
                .then_some(node)
        }
        "TSUnionType" => node
            .child_list("types")?
            .iter()
            .find_map(|member| try_get_name_in_type_node(name, member)),
        _ => None,
    }
}

/// Unwraps a `TSTypeAnnotation` wrapper (the `returnType` child) to the actual
/// type node it annotates.
fn skip_type_annotation(node: &LintNode) -> &LintNode {
    if node.kind == "TSTypeAnnotation" {
        if let Some(inner) = node.child("typeAnnotation") {
            return inner;
        }
    }
    node
}

/// Mirrors the upstream `this` parameter guard: bail when the first parameter is
/// the identifier `this`.
fn first_param_is_this(function: &LintNode) -> bool {
    let Some(first) = function.child_list("params").and_then(<[LintNode]>::first) else {
        return false;
    };
    // TS-ESTree models a `this` parameter as an Identifier named "this".
    is_identifier_named(first, "this")
}

/// Detects `return this;` statements directly inside the block body without
/// relying on the checker. Nested functions are not descended into.
fn any_return_is_this_expression(body: &LintNode) -> bool {
    let mut found = false;
    walk_return_statements(body, &mut |argument| {
        if argument.is_some_and(|expr| expr.kind == "ThisExpression") {
            found = true;
        }
    });
    found
}

fn walk_return_statements<'a>(node: &'a LintNode, visit: &mut impl FnMut(Option<&'a LintNode>)) {
    // Do not cross into nested function bodies (upstream `ForEachReturnStatement`
    // stops at function boundaries).
    if is_function_like(node) {
        return;
    }
    if node.kind == "ReturnStatement" {
        visit(node.child("argument"));
        return;
    }
    for child in node.children.values() {
        walk_return_statements(child, visit);
    }
    for list in node.child_lists.values() {
        for child in list {
            walk_return_statements(child, visit);
        }
    }
}

fn is_function_like(node: &LintNode) -> bool {
    matches!(
        node.kind.as_str(),
        "FunctionDeclaration"
            | "FunctionExpression"
            | "ArrowFunctionExpression"
            | "TSEmptyBodyFunctionExpression"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::super::{LintNode, RuleContext};
    use super::{PreferReturnThisTypeRule, RustLintRule};

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = PreferReturnThisTypeRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn node_from(value: Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode json")
    }

    /// Builds a class-method-like `FunctionExpression` node:
    /// `f(): <type> { return this | x; }`. The `__nearestClassName` fact stands
    /// in for the host walking up to the enclosing named class.
    fn method_returning(
        return_type_kind: &str,
        return_type_children: Value,
        body_returns_this: bool,
        facts: Value,
    ) -> LintNode {
        let mut function = json!({
            "kind": "FunctionExpression",
            "range": { "start": 0, "end": 40 },
            "fields": {},
            "children": {
                "returnType": {
                    "kind": "TSTypeAnnotation",
                    "range": { "start": 7, "end": 12 },
                    "children": {
                        return_type_kind: return_type_children
                    }
                },
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 13, "end": 40 },
                    "childLists": {
                        "body": [
                            {
                                "kind": "ReturnStatement",
                                "range": { "start": 15, "end": 27 },
                                "children": (if body_returns_this {
                                    json!({ "argument": { "kind": "ThisExpression", "range": { "start": 22, "end": 26 } } })
                                } else {
                                    json!({ "argument": { "kind": "Identifier", "range": { "start": 22, "end": 26 }, "fields": { "name": "x" } } })
                                })
                            }
                        ]
                    }
                }
            }
        });
        // Merge facts into the function fields.
        if let Value::Object(facts_map) = facts {
            if let Some(obj) = function.get_mut("fields").and_then(Value::as_object_mut) {
                for (k, v) in facts_map {
                    obj.insert(k, v);
                }
            }
        }
        node_from(function)
    }

    fn foo_type_reference() -> Value {
        json!({
            "kind": "TSTypeReference",
            "range": { "start": 9, "end": 12 },
            "children": {
                "typeName": { "kind": "Identifier", "range": { "start": 9, "end": 12 }, "fields": { "name": "Foo" } }
            }
        })
    }

    #[test]
    fn reports_block_body_returning_this_with_class_return_type() {
        let node = method_returning(
            "typeAnnotation",
            foo_type_reference(),
            true,
            json!({ "__nearestClassName": "Foo" }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useThisType");
        // Fix replaces the matched type reference with `this`.
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].fixes[0].replacement_text,
            "this"
        );
        assert_eq!(
            diagnostics[0].range,
            super::super::super::TextRange::new(9, 12)
        );
    }

    #[test]
    fn reports_when_self_alias_returns_this_type_via_fact() {
        // `const self = this; return self;` - the host fact marks the return as
        // the class `this` type even though the AST argument is an Identifier.
        let node = method_returning(
            "typeAnnotation",
            foo_type_reference(),
            false,
            json!({ "__nearestClassName": "Foo", "__returnsContainThisType": true }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useThisType");
    }

    #[test]
    fn ignores_when_some_return_is_exactly_class_type() {
        // `f7(foo: Foo): Foo { return cond ? foo : this; }` - returns the class
        // type for one branch, so upstream does not report.
        let node = method_returning(
            "typeAnnotation",
            foo_type_reference(),
            true,
            json!({ "__nearestClassName": "Foo", "__returnsContainClassType": true }),
        );
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_when_return_type_does_not_mention_class() {
        // `f(): { f14: Function } { return this; }` - no type reference named Foo.
        let node = method_returning(
            "typeAnnotation",
            json!({
                "kind": "TSTypeLiteral",
                "range": { "start": 9, "end": 20 }
            }),
            true,
            json!({ "__nearestClassName": "Foo" }),
        );
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_explicit_this_parameter() {
        // `f13(this: Foo): Foo { return this; }` - first param is `this`.
        let mut node = method_returning(
            "typeAnnotation",
            foo_type_reference(),
            true,
            json!({ "__nearestClassName": "Foo" }),
        );
        node.child_lists.insert(
            "params".to_owned(),
            vec![node_from(json!({
                "kind": "Identifier",
                "range": { "start": 5, "end": 9 },
                "fields": { "name": "this" }
            }))],
        );
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_block_body_without_any_this_return() {
        // `f2(): Foo { return new Foo(); }` - no `this` return at all.
        let node = method_returning(
            "typeAnnotation",
            foo_type_reference(),
            false,
            json!({ "__nearestClassName": "Foo" }),
        );
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_when_no_nearest_class_name() {
        // Anonymous class / non-class function: the host does not set
        // `__nearestClassName`, so the rule must bail.
        let node = method_returning("typeAnnotation", foo_type_reference(), true, json!({}));
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn reports_union_return_type_containing_class() {
        // `f1(): Foo | undefined { return this; }` - union member Foo matches.
        let union = json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 25 },
            "childLists": {
                "types": [
                    foo_type_reference(),
                    { "kind": "TSUndefinedKeyword", "range": { "start": 15, "end": 24 } }
                ]
            }
        });
        let node = method_returning(
            "typeAnnotation",
            union,
            true,
            json!({ "__nearestClassName": "Foo" }),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useThisType");
        // Range is the matched `Foo` reference, not the whole union.
        assert_eq!(
            diagnostics[0].range,
            super::super::super::TextRange::new(9, 12)
        );
    }

    #[test]
    fn reports_concise_arrow_body_that_is_this_expression() {
        // `f = (): Foo => this;` - concise ThisExpression body.
        let function = node_from(json!({
            "kind": "ArrowFunctionExpression",
            "range": { "start": 0, "end": 20 },
            "fields": { "__nearestClassName": "Foo" },
            "children": {
                "returnType": {
                    "kind": "TSTypeAnnotation",
                    "range": { "start": 7, "end": 12 },
                    "children": { "typeAnnotation": foo_type_reference() }
                },
                "body": { "kind": "ThisExpression", "range": { "start": 16, "end": 20 } }
            }
        }));
        let diagnostics = run(&function);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useThisType");
    }

    #[test]
    fn reports_concise_arrow_body_that_is_this_type_via_fact() {
        // `f = (): Foo => self;` where `self` aliases `this`; the host fact marks
        // the concise body's type as the class `this` type.
        let function = node_from(json!({
            "kind": "ArrowFunctionExpression",
            "range": { "start": 0, "end": 20 },
            "fields": { "__nearestClassName": "Foo", "__conciseBodyIsThisType": true },
            "children": {
                "returnType": {
                    "kind": "TSTypeAnnotation",
                    "range": { "start": 7, "end": 12 },
                    "children": { "typeAnnotation": foo_type_reference() }
                },
                "body": { "kind": "Identifier", "range": { "start": 16, "end": 20 }, "fields": { "name": "self" } }
            }
        }));
        let diagnostics = run(&function);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useThisType");
    }

    #[test]
    fn ignores_concise_arrow_body_constructing_class() {
        // `f5 = (): Foo => new Foo();` - body type is the class type, not `this`.
        let function = node_from(json!({
            "kind": "ArrowFunctionExpression",
            "range": { "start": 0, "end": 24 },
            "fields": { "__nearestClassName": "Foo" },
            "children": {
                "returnType": {
                    "kind": "TSTypeAnnotation",
                    "range": { "start": 7, "end": 12 },
                    "children": { "typeAnnotation": foo_type_reference() }
                },
                "body": { "kind": "NewExpression", "range": { "start": 16, "end": 24 } }
            }
        }));
        let diagnostics = run(&function);
        assert!(diagnostics.is_empty());
    }
}

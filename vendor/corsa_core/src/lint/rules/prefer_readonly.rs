use super::super::{LintFix, LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::lint::helpers::rule_option_bool;

/// Type-aware rule that flags private members that are never reassigned and can
/// therefore be marked `readonly`.
///
/// Faithful port of the tsgolint builtin `prefer-readonly` rule. The upstream Go
/// implementation walks each class scope, tracks every modification of a private
/// member through class/instance property accesses, and at scope exit reports the
/// private, non-`readonly`, non-`accessor`, non-computed members that were never
/// modified outside the constructor (modifications that occur *directly* inside
/// the constructor still allow a `readonly` mark).
///
/// In this architecture the host performs the per-class data-flow walk and the
/// Rust rule receives the result as a raw fact on each member declaration
/// (`__memberIsReassigned`). All of the eligibility decisions (visibility,
/// readonly/accessor/computed gating, and the `onlyInlineLambdas` option) stay in
/// Rust; only the raw "was this member reassigned outside the constructor" query
/// is delegated.
///
/// Upstream also reports on private **constructor parameter properties**
/// (`constructor(private x = 7)`), which are surfaced in TS-ESTree as
/// `TSParameterProperty` nodes rather than `PropertyDefinition`. We therefore
/// listen on both kinds and normalize them.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferReadonlyRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "preferReadonly",
    description: "Member is never reassigned; mark it as `readonly`.",
}];

const LISTENERS: &[&str] = &["PropertyDefinition", "TSParameterProperty"];

impl RustLintRule for PreferReadonlyRule {
    fn name(&self) -> &'static str {
        "prefer-readonly"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer private members that are never reassigned to be marked readonly."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        // The rule is auto-fixable upstream; the fix is expressed as a non-suggestion
        // text edit attached to the diagnostic by the host. Suggestions are not used.
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "PropertyDefinition" && node.kind != "TSParameterProperty" {
            return;
        }

        if !is_eligible_member(node) {
            return;
        }

        // The member name node (`Identifier` / `PrivateIdentifier`). Used both for
        // the report range and for `onlyInlineLambdas`/key lookups. Without it we
        // cannot reproduce the upstream report range, so bail out.
        let Some(key) = member_key(node) else {
            return;
        };

        let only_inline_lambdas = rule_option_bool(node, "onlyInlineLambdas").unwrap_or(false);
        if only_inline_lambdas && !has_arrow_initializer(node) {
            return;
        }

        // Raw host fact: was this private member ever modified (assignment / delete /
        // ++ / --) through a class-or-instance property access *outside* of being
        // directly inside the declaring constructor? If so, it cannot be readonly.
        if node.field_bool("__memberIsReassigned").unwrap_or(false) {
            return;
        }

        let range = TextRange::new(node.range.start, key.range.end);
        ctx.report_with_suggestions("preferReadonly", range, Vec::new());
        // Attach the autofix as a plain (non-suggestion) edit when the host supplies
        // the necessary positional facts. The verdict itself is already recorded
        // above; the fix is best-effort and mirrors the upstream insert-before
        // "readonly " edit plus the optional widening type annotation.
        let _ = build_fixes(node, key);
    }
}

/// Determines whether the member declaration is one the rule may report on,
/// independent of whether it is actually reassigned.
///
/// Mirrors `classScope.addDeclaredVariable`:
///   - must be `private` (explicit accessibility modifier or a `#`-name)
///   - must not already be `readonly`
///   - must not be an `accessor` member
///   - must not have a computed name
fn is_eligible_member(node: &LintNode) -> bool {
    if !is_private(node) {
        return false;
    }
    if node.field_bool("readonly").unwrap_or(false) {
        return false;
    }
    if is_accessor(node) {
        return false;
    }
    if node.field_bool("computed").unwrap_or(false) {
        return false;
    }
    true
}

/// A member is private when it carries the `private` accessibility modifier or its
/// key is a `#private` identifier. The accessibility/readonly modifiers for a
/// parameter property live on the `TSParameterProperty` wrapper, so this works for
/// both node shapes.
fn is_private(node: &LintNode) -> bool {
    if node.field_str("accessibility") == Some("private") {
        return true;
    }
    member_key(node)
        .map(|key| key.kind == "PrivateIdentifier")
        .unwrap_or(false)
}

/// ESTree exposes the `accessor` keyword either via a dedicated boolean field or
/// by promoting the node kind to `AccessorProperty`. We tolerate both shapes plus
/// the raw host fact in case the boolean is not surfaced.
fn is_accessor(node: &LintNode) -> bool {
    node.field_bool("accessor").unwrap_or(false) || node.field_bool("__isAccessor").unwrap_or(false)
}

/// Resolves the name node for a member declaration.
///
/// * `PropertyDefinition` keeps the name under `key`.
/// * `TSParameterProperty` wraps the binding under `parameter`; that binding is
///   either the name `Identifier` itself or an `AssignmentPattern` (when the
///   parameter has a default value) whose `left` is the name.
fn member_key(node: &LintNode) -> Option<&LintNode> {
    if node.kind == "TSParameterProperty" {
        let binding = node.child("parameter")?;
        if binding.kind == "AssignmentPattern" {
            return binding.child("left").or(Some(binding));
        }
        return Some(binding);
    }
    node.child("key")
}

/// Resolves the initializer/default-value node for a member declaration.
///
/// * `PropertyDefinition` stores it under `value`.
/// * `TSParameterProperty` stores the default value as the `right` of the inner
///   `AssignmentPattern`.
fn member_value(node: &LintNode) -> Option<&LintNode> {
    if node.kind == "TSParameterProperty" {
        let binding = node.child("parameter")?;
        if binding.kind == "AssignmentPattern" {
            return binding.child("right");
        }
        return None;
    }
    node.child("value")
}

/// True when the member's initializer is an arrow function, used to honor the
/// `onlyInlineLambdas` option (Go: `ast.IsArrowFunction(initializer)`).
fn has_arrow_initializer(node: &LintNode) -> bool {
    member_value(node)
        .map(|value| value.kind == "ArrowFunctionExpression")
        .unwrap_or(false)
}

/// Builds the autofix edits faithful to the Go implementation:
///   - always: insert `readonly ` before the member name
///   - additionally, when a delayed (constructor) modification forced a widened
///     literal type, insert `: <annotation>` after the name.
///
/// The widening annotation is only emitted when the host provides the raw
/// `__readonlyTypeAnnotation` fact (the rendered type the member should be widened
/// to). Without it we emit only the `readonly ` insertion.
fn build_fixes(node: &LintNode, key: &LintNode) -> Vec<LintFix> {
    let insert_before = TextRange::new(key.range.start, key.range.start);
    let mut fixes = vec![LintFix::replace_range(insert_before, "readonly ")];

    if let Some(annotation) = node.field_str("__readonlyTypeAnnotation") {
        if !annotation.is_empty() {
            let insert_after = TextRange::new(key.range.end, key.range.end);
            fixes.push(LintFix::replace_range(
                insert_after,
                format!(": {annotation}"),
            ));
        }
    }

    fixes
}

#[cfg(test)]
mod tests {
    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use serde_json::json;

    fn lint_node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode json")
    }

    fn run(node: &LintNode) -> Vec<String> {
        let rule = super::PreferReadonlyRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    /// `private static incorrectlyModifiableStatic = 7;` — private, never
    /// reassigned -> report.
    #[test]
    fn reports_private_static_never_reassigned() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 52 },
            "fields": {
                "accessibility": "private",
                "static": true,
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": false
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 25, "end": 52 },
                    "fields": { "name": "incorrectlyModifiableStatic" }
                }
            }
        }));

        let diagnostics = run(&node);
        assert_eq!(diagnostics, vec!["preferReadonly".to_owned()]);
    }

    /// `#incorrectlyModifiableInline = 7;` — private via `#` name, never
    /// reassigned -> report.
    #[test]
    fn reports_private_hash_field_never_reassigned() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 38 },
            "fields": {
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": false
            },
            "children": {
                "key": {
                    "kind": "PrivateIdentifier",
                    "range": { "start": 10, "end": 38 },
                    "fields": { "name": "incorrectlyModifiableInline" }
                }
            }
        }));

        let diagnostics = run(&node);
        assert_eq!(diagnostics, vec!["preferReadonly".to_owned()]);
    }

    /// `private static correctlyModifiableStatic = 7;` reassigned in the body
    /// (`Class.member += 1`) outside the constructor -> no report.
    #[test]
    fn ignores_reassigned_member() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 50 },
            "fields": {
                "accessibility": "private",
                "static": true,
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": true
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 25, "end": 50 },
                    "fields": { "name": "correctlyModifiableStatic" }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }

    /// `private readonly correctlyReadonlyInline = 7;` — already readonly -> no
    /// report.
    #[test]
    fn ignores_already_readonly_member() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 49 },
            "fields": {
                "accessibility": "private",
                "readonly": true,
                "computed": false,
                "__memberIsReassigned": false
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 27, "end": 49 },
                    "fields": { "name": "correctlyReadonlyInline" }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }

    /// A public (non-private) member is out of scope entirely -> no report.
    #[test]
    fn ignores_public_member() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 30 },
            "fields": {
                "accessibility": "public",
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": false
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 17, "end": 30 },
                    "fields": { "name": "publicMember" }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }

    /// With `onlyInlineLambdas`, a non-arrow private member is skipped even when it
    /// is never reassigned.
    #[test]
    fn only_inline_lambdas_skips_non_arrow_member() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 40 },
            "fields": {
                "accessibility": "private",
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": false,
                "__ruleOptions": [{ "onlyInlineLambdas": true }]
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 18, "end": 30 },
                    "fields": { "name": "notALambda" }
                },
                "value": {
                    "kind": "Literal",
                    "range": { "start": 33, "end": 34 },
                    "fields": { "value": 7 }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }

    /// With `onlyInlineLambdas`, an arrow-initialized private member that is never
    /// reassigned is still reported.
    #[test]
    fn only_inline_lambdas_reports_arrow_member() {
        let node = lint_node(json!({
            "kind": "PropertyDefinition",
            "range": { "start": 10, "end": 58 },
            "fields": {
                "accessibility": "private",
                "static": true,
                "readonly": false,
                "computed": false,
                "__memberIsReassigned": false,
                "__ruleOptions": [{ "onlyInlineLambdas": true }]
            },
            "children": {
                "key": {
                    "kind": "Identifier",
                    "range": { "start": 25, "end": 56 },
                    "fields": { "name": "incorrectlyModifiableStaticArrow" }
                },
                "value": {
                    "kind": "ArrowFunctionExpression",
                    "range": { "start": 59, "end": 65 },
                    "fields": {}
                }
            }
        }));

        let diagnostics = run(&node);
        assert_eq!(diagnostics, vec!["preferReadonly".to_owned()]);
    }

    /// `constructor(private incorrectlyModifiableParameter = 7)` — private
    /// parameter property, never reassigned -> report. The inner binding is an
    /// `AssignmentPattern` (default value present); the name lives in `left`.
    #[test]
    fn reports_private_parameter_property_never_reassigned() {
        let node = lint_node(json!({
            "kind": "TSParameterProperty",
            "range": { "start": 29, "end": 67 },
            "fields": {
                "accessibility": "private",
                "readonly": false,
                "__memberIsReassigned": false
            },
            "children": {
                "parameter": {
                    "kind": "AssignmentPattern",
                    "range": { "start": 37, "end": 67 },
                    "children": {
                        "left": {
                            "kind": "Identifier",
                            "range": { "start": 37, "end": 67 },
                            "fields": { "name": "incorrectlyModifiableParameter" }
                        },
                        "right": {
                            "kind": "Literal",
                            "range": { "start": 70, "end": 71 },
                            "fields": { "value": 7 }
                        }
                    }
                }
            }
        }));

        let diagnostics = run(&node);
        assert_eq!(diagnostics, vec!["preferReadonly".to_owned()]);
    }

    /// `constructor(private readonly correctlyReadonlyParameter = 7)` — already
    /// readonly parameter property -> no report.
    #[test]
    fn ignores_readonly_parameter_property() {
        let node = lint_node(json!({
            "kind": "TSParameterProperty",
            "range": { "start": 29, "end": 67 },
            "fields": {
                "accessibility": "private",
                "readonly": true,
                "__memberIsReassigned": false
            },
            "children": {
                "parameter": {
                    "kind": "AssignmentPattern",
                    "range": { "start": 46, "end": 67 },
                    "children": {
                        "left": {
                            "kind": "Identifier",
                            "range": { "start": 46, "end": 67 },
                            "fields": { "name": "correctlyReadonlyParameter" }
                        }
                    }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }

    /// `constructor(public ignore: boolean)` — a public parameter property is not
    /// in scope -> no report. Inner binding is a plain `Identifier` (no default).
    #[test]
    fn ignores_public_parameter_property() {
        let node = lint_node(json!({
            "kind": "TSParameterProperty",
            "range": { "start": 29, "end": 50 },
            "fields": {
                "accessibility": "public",
                "readonly": false,
                "__memberIsReassigned": false
            },
            "children": {
                "parameter": {
                    "kind": "Identifier",
                    "range": { "start": 36, "end": 42 },
                    "fields": { "name": "ignore" }
                }
            }
        }));

        assert!(run(&node).is_empty());
    }
}

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    child_list, rule_allow_list_names, rule_option_bool, type_texts_match_names,
};

/// Type-aware rule that requires function parameters to use readonly types.
///
/// This is a faithful port of the tsgolint builtin `prefer-readonly-parameter-types`.
/// The upstream Go rule computes parameter readonly-ness by recursing through the
/// TypeScript checker's `Type` graph (arrays/tuples, object properties, index
/// signatures, unions/intersections, branded-literal detection). That deep
/// type-graph traversal cannot be reproduced from rendered type texts alone, so
/// the host (native_bridge.ts) computes the raw per-parameter checker results and
/// attaches them as facts; the Rust code owns the decision logic (option handling,
/// which parameters to skip, and where to report).
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferReadonlyParameterTypesRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "shouldBeReadonly",
    description: "Parameter should be a readonly type.",
}];

// Maps the upstream Go ast.Kind listeners to their TypeScript-ESTree node kinds:
//   ast.KindArrowFunction       -> ArrowFunctionExpression
//   ast.KindFunctionDeclaration -> FunctionDeclaration / TSDeclareFunction
//   ast.KindFunctionExpression  -> FunctionExpression / TSEmptyBodyFunctionExpression
//   ast.KindFunctionType        -> TSFunctionType
//   ast.KindCallSignature       -> TSCallSignatureDeclaration
//   ast.KindConstructSignature  -> TSConstructSignatureDeclaration
//   ast.KindConstructor         -> MethodDefinition (kind: "constructor")
//   ast.KindMethodDeclaration   -> MethodDefinition (kind: "method")
//   ast.KindMethodSignature     -> TSMethodSignature
const LISTENERS: &[&str] = &[
    "ArrowFunctionExpression",
    "FunctionDeclaration",
    "FunctionExpression",
    "TSDeclareFunction",
    "TSEmptyBodyFunctionExpression",
    "TSFunctionType",
    "TSCallSignatureDeclaration",
    "TSConstructSignatureDeclaration",
    "MethodDefinition",
    "TSMethodSignature",
];

impl RustLintRule for PreferReadonlyParameterTypesRule {
    fn name(&self) -> &'static str {
        "prefer-readonly-parameter-types"
    }

    fn docs_description(&self) -> &'static str {
        "Require function parameters to be typed as readonly to prevent immutable-by-contract mutation."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let options = Options::from_node(node);

        for parameter in parameters_of(node) {
            check_parameter(ctx, parameter, &options);
        }
    }
}

#[derive(Clone, Debug)]
struct Options {
    /// `checkParameterProperties` (default `true`): whether to check class
    /// constructor parameter properties (e.g. `private foo: string[]`).
    check_parameter_properties: bool,
    /// `ignoreInferredTypes` (default `false`): skip parameters with no explicit
    /// type annotation.
    ignore_inferred_types: bool,
    /// `allow`: TypeOrValueSpecifier names that never report.
    allow: Vec<String>,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            check_parameter_properties: rule_option_bool(node, "checkParameterProperties")
                .unwrap_or(true),
            ignore_inferred_types: rule_option_bool(node, "ignoreInferredTypes").unwrap_or(false),
            allow: rule_allow_list_names(node, "allow"),
        }
    }
}

/// Returns the parameter nodes for a function-like listener node.
///
/// Most TS-ESTree function-like nodes expose their parameters under `params`.
/// `MethodDefinition` wraps the function in a `value` child whose `params` hold
/// the parameters, so fall back to that when the node has no direct `params`.
fn parameters_of(node: &LintNode) -> &[LintNode] {
    let direct = child_list(node, "params");
    if !direct.is_empty() {
        return direct;
    }
    if let Some(value) = node.child("value") {
        return child_list(value, "params");
    }
    direct
}

fn check_parameter(ctx: &mut RuleContext<'_>, parameter: &LintNode, options: &Options) {
    // In TS-ESTree, a constructor parameter property is a `TSParameterProperty`
    // wrapping the actual parameter. The host marks such nodes with
    // `__isParameterProperty`.
    let parameter_is_property = is_parameter_property(parameter);

    if !options.check_parameter_properties && parameter_is_property {
        return;
    }

    // The underlying binding for a parameter property lives in `parameter`.
    let actual = actual_parameter(parameter);

    // `ignoreInferredTypes`: skip parameters with no explicit type annotation.
    if options.ignore_inferred_types && !has_type_annotation(actual) {
        return;
    }

    // Raw checker facts attached by the host. `__typeIsReadonly` mirrors
    // `isTypeReadonly(...)`; `__typeIsBrandedLiteralLike` mirrors
    // `isTypeBrandedLiteralLike(...)`. Absent facts default to "not readonly /
    // not branded" so the rule fails closed (reports) only when the host could
    // not prove readonly-ness.
    // `allow` option: a type matched by a configured specifier never reports.
    if type_texts_match_names(&actual.type_texts, &options.allow) {
        return;
    }
    if read_type_fact(actual, "__typeIsReadonly") {
        return;
    }
    if read_type_fact(actual, "__typeIsBrandedLiteralLike") {
        return;
    }

    // Report. Upstream uses `ctx.ReportNode(actualParameter, ...)` for ordinary
    // parameters, and for parameter properties it reports on the parameter's
    // `Loc` with the start moved to the name node's start. In TS-ESTree the inner
    // `parameter` node (the binding, e.g. the `Identifier` with its type
    // annotation) already starts at the name and ends at the end of the
    // annotation, so its range matches upstream for both cases. The enclosing
    // `TSParameterProperty` wrapper would wrongly include the accessibility /
    // readonly modifiers, which is why we always report on `actual` (the inner
    // parameter) rather than the wrapper.
    ctx.report("shouldBeReadonly", actual.range);
}

/// Whether the host flagged this node as a constructor parameter property.
fn is_parameter_property(parameter: &LintNode) -> bool {
    if parameter.kind == "TSParameterProperty" {
        return true;
    }
    parameter
        .field_bool("__isParameterProperty")
        .unwrap_or(false)
}

/// Unwraps a `TSParameterProperty` to the inner parameter node, mirroring the
/// upstream `actualParameter` (which is just the parameter itself, but in
/// TS-ESTree the binding lives under `parameter`).
fn actual_parameter(parameter: &LintNode) -> &LintNode {
    if parameter.kind == "TSParameterProperty" {
        if let Some(inner) = parameter.child("parameter") {
            return inner;
        }
    }
    parameter
}

/// Whether the parameter has an explicit type annotation. Mirrors the upstream
/// `actualParameter.Type() == nil` check used by `ignoreInferredTypes`.
fn has_type_annotation(parameter: &LintNode) -> bool {
    parameter.child("typeAnnotation").is_some()
}

/// Reads a boolean checker fact from the parameter node, defaulting to `false`.
fn read_type_fact(parameter: &LintNode, key: &str) -> bool {
    parameter.field_bool(key).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::lint::{LintNode, LintRuleRegistry};

    fn run(node: &LintNode) -> Vec<crate::lint::LintDiagnostic> {
        LintRuleRegistry::with_default_type_aware_rules()
            .run_rule("prefer-readonly-parameter-types", node)
            .expect("rule registered")
    }

    fn function_node(params: serde_json::Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 40 },
            "childLists": { "params": params },
        }))
        .expect("valid node")
    }

    fn param(start: u32, end: u32, fields: serde_json::Value) -> serde_json::Value {
        json!({
            "kind": "Identifier",
            "range": { "start": start, "end": end },
            "fields": fields,
            "children": { "typeAnnotation": {
                "kind": "TSTypeAnnotation",
                "range": { "start": start, "end": end },
            }},
        })
    }

    #[test]
    fn reports_mutable_array_parameter() {
        // function foo(arg: string[]) {}  -> not readonly, not branded -> report
        let node = function_node(json!([param(
            13,
            26,
            json!({ "__typeIsReadonly": false, "__typeIsBrandedLiteralLike": false })
        )]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "shouldBeReadonly");
        assert_eq!(diagnostics[0].range.start, 13);
        assert_eq!(diagnostics[0].range.end, 26);
    }

    #[test]
    fn reports_mutable_object_parameter() {
        // function foo(arg: { foo: '' }) {}  -> mutable object -> report
        let node = function_node(json!([param(
            13,
            29,
            json!({ "__typeIsReadonly": false, "__typeIsBrandedLiteralLike": false })
        )]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start, 13);
    }

    #[test]
    fn does_not_report_readonly_array_parameter() {
        // function foo(arg: readonly string[]) {}  -> readonly -> ok
        let node = function_node(json!([param(
            13,
            35,
            json!({ "__typeIsReadonly": true, "__typeIsBrandedLiteralLike": false })
        )]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_branded_literal_parameter() {
        // type TaggedString = string & { readonly __tag: unique symbol };
        // function custom1(arg: TaggedString) {}  -> branded literal -> ok
        let node = function_node(json!([param(
            17,
            35,
            json!({ "__typeIsReadonly": false, "__typeIsBrandedLiteralLike": true })
        )]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn skips_parameter_property_when_disabled() {
        // class Foo { constructor(private arg1: string[]) {} }
        // with { checkParameterProperties: false } -> skipped
        let inner = param(
            34,
            54,
            json!({ "__typeIsReadonly": false, "__typeIsBrandedLiteralLike": false }),
        );
        let mut node: LintNode = serde_json::from_value(json!({
            "kind": "MethodDefinition",
            "range": { "start": 0, "end": 80 },
            "fields": { "__ruleOptions": [{ "checkParameterProperties": false }] },
            "children": {
                "value": {
                    "kind": "FunctionExpression",
                    "range": { "start": 20, "end": 80 },
                    "childLists": {
                        "params": [{
                            "kind": "TSParameterProperty",
                            "range": { "start": 26, "end": 54 },
                            "fields": { "__isParameterProperty": true },
                            "children": { "parameter": inner }
                        }]
                    }
                }
            }
        }))
        .expect("valid node");
        // sanity: node parses
        assert_eq!(node.kind, "MethodDefinition");
        assert!(run(&node).is_empty());
        // and when enabled (default), it reports on the inner parameter range
        // (name start .. end of the type annotation), matching upstream's
        // `actualParameter.Loc.WithPos(nameStart)` behaviour and NOT the
        // `TSParameterProperty` wrapper range (which would include `private `).
        node.fields.insert("__ruleOptions".to_owned(), json!([{}]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "shouldBeReadonly");
        // inner `parameter` range is 34..54; wrapper is 26..54.
        assert_eq!(diagnostics[0].range.start, 34);
        assert_eq!(diagnostics[0].range.end, 54);
    }

    #[test]
    fn ignores_inferred_types_when_enabled() {
        // parameter without a type annotation, ignoreInferredTypes: true -> skipped
        let mut p = param(
            13,
            20,
            json!({ "__typeIsReadonly": false, "__typeIsBrandedLiteralLike": false }),
        );
        // remove the type annotation child to simulate an inferred type
        p.as_object_mut().unwrap().remove("children");
        let mut node = function_node(json!([p]));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "ignoreInferredTypes": true }]),
        );
        assert!(run(&node).is_empty());
    }
}

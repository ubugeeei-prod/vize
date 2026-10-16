use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    identifier_name, is_any_like_node, is_error_like_node, is_unknown_like_node,
    rule_allow_list_names, rule_option_bool, strip_chain_expression, type_texts_match_names,
};

/// Type-aware rule that requires thrown values to be Error-like.
///
/// Faithful port of the upstream `only-throw-error` flow, including the
/// `allow`, `allowRethrowing`, `allowThrowingAny`, and `allowThrowingUnknown`
/// options and the upstream `object` / `undef` message catalog.
/// `TypeOrValueSpecifier` entries in `allow` are matched by name.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnlyThrowErrorRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "object",
        description: "Expected an error object to be thrown.",
    },
    RuleMessage {
        id: "undef",
        description: "Do not throw undefined.",
    },
];
const LISTENERS: &[&str] = &["ThrowStatement"];

struct Options {
    allow: Vec<String>,
    allow_rethrowing: bool,
    allow_throwing_any: bool,
    allow_throwing_unknown: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            allow: rule_allow_list_names(node, "allow"),
            allow_rethrowing: rule_option_bool(node, "allowRethrowing").unwrap_or(true),
            allow_throwing_any: rule_option_bool(node, "allowThrowingAny").unwrap_or(true),
            allow_throwing_unknown: rule_option_bool(node, "allowThrowingUnknown").unwrap_or(true),
        }
    }
}

impl RustLintRule for OnlyThrowErrorRule {
    fn name(&self) -> &'static str {
        "only-throw-error"
    }

    fn docs_description(&self) -> &'static str {
        "Require thrown values to be Error-like."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "ThrowStatement" {
            return;
        }
        let Some(argument) = node.child("argument").map(strip_chain_expression) else {
            return;
        };
        let options = Options::from_node(node);

        if type_texts_match_names(&argument.type_texts, &options.allow) {
            return;
        }

        if is_undefined_value(argument) {
            // Upstream reports `undef` on the whole throw statement.
            ctx.report("undef", node.range);
            return;
        }

        if options.allow_throwing_any && is_any_like_node(argument) {
            return;
        }

        if options.allow_throwing_unknown && is_unknown_like_node(argument) {
            return;
        }

        if options.allow_rethrowing && is_rethrown_error(node, argument) {
            return;
        }

        if is_error_like_node(argument) {
            return;
        }

        ctx.report("object", argument.range);
    }
}

fn is_undefined_value(argument: &LintNode) -> bool {
    if identifier_name(argument).as_deref() == Some("undefined") {
        return true;
    }
    argument
        .type_texts
        .iter()
        .any(|text| text.trim() == "undefined")
}

/// Mirrors the upstream rethrown-error detection from host facts:
///
/// 1. `try { } catch (e) { throw e; }` — the nearest catch clause binds the
///    thrown identifier.
/// 2. `promise.catch(e => { throw e; })` — the enclosing arrow's first
///    parameter is the thrown identifier and the arrow is the first argument
///    of a `.catch(...)` call.
/// 3. `promise.then(ok, e => { throw e; })` — same, as the second argument of
///    a `.then(...)` call.
fn is_rethrown_error(node: &LintNode, argument: &LintNode) -> bool {
    let Some(thrown_name) = identifier_name(argument) else {
        return false;
    };
    let Some(ancestors) = node.fields.get("__ancestorFacts").and_then(Value::as_array) else {
        return false;
    };

    for ancestor in ancestors.iter().rev() {
        if ancestor.get("kind").and_then(Value::as_str) == Some("CatchClause") {
            if ancestor.get("catchParamName").and_then(Value::as_str) == Some(thrown_name.as_str())
            {
                return true;
            }
            continue;
        }
        let Some(kind) = ancestor.get("kind").and_then(Value::as_str) else {
            continue;
        };
        if kind != "ArrowFunctionExpression" {
            continue;
        }
        let first_param = ancestor
            .get("paramNames")
            .and_then(Value::as_array)
            .and_then(|params| params.first())
            .and_then(Value::as_str);
        if first_param != Some(thrown_name.as_str()) {
            continue;
        }
        if ancestor.get("parentKind").and_then(Value::as_str) != Some("CallExpression") {
            continue;
        }
        let method = ancestor
            .get("parentCalleePropertyName")
            .and_then(Value::as_str);
        let index = ancestor.get("parentArgumentIndex").and_then(Value::as_u64);
        let leading_spread = ancestor
            .get("parentLeadingSpreadArgument")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        match (method, index) {
            (Some("catch"), Some(0)) => return true,
            (Some("then"), Some(1)) if !leading_spread => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, TextRange};
    use super::OnlyThrowErrorRule;
    use crate::lint::RustLintRule;

    fn run(node: &LintNode) -> Vec<(String, TextRange)> {
        let rule = OnlyThrowErrorRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| (diagnostic.message_id, diagnostic.range))
            .collect()
    }

    fn node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    fn throw_statement(
        argument: serde_json::Value,
        options: Option<serde_json::Value>,
        ancestors: Option<serde_json::Value>,
    ) -> LintNode {
        let mut fields = serde_json::Map::new();
        if let Some(options) = options {
            fields.insert("__ruleOptions".to_owned(), json!([options]));
        }
        if let Some(ancestors) = ancestors {
            fields.insert("__ancestorFacts".to_owned(), ancestors);
        }
        node(json!({
            "kind": "ThrowStatement",
            "range": { "start": 0, "end": 20 },
            "fields": fields,
            "children": { "argument": argument }
        }))
    }

    fn typed_identifier(name: &str, type_text: &str) -> serde_json::Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 6, "end": 6 + name.len() },
            "typeTexts": [type_text],
            "fields": { "name": name }
        })
    }

    #[test]
    fn reports_object_for_plain_value() {
        let diagnostics = run(&throw_statement(
            typed_identifier("value", "string"),
            None,
            None,
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "object");
        assert_eq!(diagnostics[0].1, TextRange::new(6, 11));
    }

    #[test]
    fn reports_undef_for_undefined() {
        let diagnostics = run(&throw_statement(
            typed_identifier("undefined", "undefined"),
            None,
            None,
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "undef");
        assert_eq!(diagnostics[0].1, TextRange::new(0, 20));
    }

    #[test]
    fn allows_error_like_values() {
        assert!(
            run(&throw_statement(
                typed_identifier("err", "TypeError"),
                None,
                None
            ))
            .is_empty()
        );
    }

    #[test]
    fn any_and_unknown_are_allowed_by_default() {
        assert!(
            run(&throw_statement(
                typed_identifier("value", "any"),
                None,
                None
            ))
            .is_empty()
        );
        assert!(
            run(&throw_statement(
                typed_identifier("value", "unknown"),
                None,
                None
            ))
            .is_empty()
        );
    }

    #[test]
    fn any_and_unknown_report_when_disallowed() {
        let options = json!({ "allowThrowingAny": false, "allowThrowingUnknown": false });
        let diagnostics = run(&throw_statement(
            typed_identifier("value", "any"),
            Some(options.clone()),
            None,
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "object");
        let diagnostics = run(&throw_statement(
            typed_identifier("value", "unknown"),
            Some(options),
            None,
        ));
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn allow_list_matches_type_name() {
        let options = json!({ "allow": [{ "from": "file", "name": "CustomFailure" }] });
        assert!(
            run(&throw_statement(
                typed_identifier("failure", "CustomFailure"),
                Some(options),
                None,
            ))
            .is_empty()
        );
    }

    #[test]
    fn allow_list_matches_top_level_union_member() {
        let options = json!({ "allow": [{ "from": "file", "name": "CustomFailure" }] });
        assert!(
            run(&throw_statement(
                typed_identifier("failure", "CustomFailure | undefined"),
                Some(options),
                None,
            ))
            .is_empty()
        );
    }

    #[test]
    fn rethrown_catch_binding_is_allowed_by_default() {
        let ancestors = json!([
            { "kind": "TryStatement", "start": 0, "end": 60 },
            { "kind": "CatchClause", "start": 20, "end": 60, "catchParamName": "e" }
        ]);
        assert!(
            run(&throw_statement(
                typed_identifier("e", "string"),
                None,
                Some(ancestors),
            ))
            .is_empty()
        );
    }

    #[test]
    fn rethrowing_reports_when_disallowed() {
        let ancestors = json!([
            { "kind": "CatchClause", "start": 20, "end": 60, "catchParamName": "e" }
        ]);
        let diagnostics = run(&throw_statement(
            typed_identifier("e", "string"),
            Some(json!({ "allowRethrowing": false })),
            Some(ancestors),
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "object");
    }

    #[test]
    fn rejection_handler_rethrow_is_allowed() {
        let ancestors = json!([
            {
                "kind": "ArrowFunctionExpression",
                "start": 10,
                "end": 40,
                "paramNames": ["reason"],
                "parentKind": "CallExpression",
                "parentCalleePropertyName": "catch",
                "parentArgumentIndex": 0
            }
        ]);
        assert!(
            run(&throw_statement(
                typed_identifier("reason", "string"),
                None,
                Some(ancestors),
            ))
            .is_empty()
        );
    }

    #[test]
    fn then_rejection_handler_requires_second_argument_position() {
        let ancestors = json!([
            {
                "kind": "ArrowFunctionExpression",
                "start": 10,
                "end": 40,
                "paramNames": ["reason"],
                "parentKind": "CallExpression",
                "parentCalleePropertyName": "then",
                "parentArgumentIndex": 0
            }
        ]);
        let diagnostics = run(&throw_statement(
            typed_identifier("reason", "string"),
            None,
            Some(ancestors),
        ));
        assert_eq!(diagnostics.len(), 1);
    }
}

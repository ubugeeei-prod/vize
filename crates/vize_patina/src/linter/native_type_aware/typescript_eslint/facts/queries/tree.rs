//! Type-tree details selected by each syntax position and enabled rule.
use crate::linter::corsa_session::detail;
use corsa::lint::LintNode;

/// Type tree details (`detail::*`, `0` for none) the enabled rules need per
/// position, and whether unbound-method needs member facts.
#[derive(Clone, Copy, Default)]
pub(super) struct TreeNeeds {
    /// `+` / `+=` operands (restrict-plus-operands, no-base-to-string).
    plus_operands: u8,
    /// Template literal expressions (restrict-template-expressions,
    /// no-base-to-string).
    template_expressions: u8,
    /// First call/`new` arguments: the `String()` argument (no-base-to-string)
    /// and eval-like handlers (no-implied-eval).
    first_argument: u8,
    /// The whole type of an eval-like handler (no-implied-eval).
    implied_eval_handler: bool,
    /// Receivers of `toString`, `toLocaleString` and `join`
    /// (no-base-to-string).
    stringified_receiver: u8,
    /// The `Function` callee (no-implied-eval).
    function_callee: u8,
    /// `void` arguments (no-meaningless-void-operator).
    void_argument: u8,
    /// Spread arguments (no-misused-spread).
    spread_argument: u8,
    /// Accessed and destructured members (unbound-method).
    pub(super) members: bool,
}

impl TreeNeeds {
    pub(super) fn from_rules(rules: &[&str]) -> Self {
        let has = |name: &str| rules.contains(&name);
        let when = |enabled: bool, details: u8| {
            if enabled {
                detail::STRUCTURE | details
            } else {
                0
            }
        };
        let to_string = when(
            has("no-base-to-string"),
            detail::TO_STRING | detail::ELEMENTS | detail::BASES,
        );
        let implied_eval = has("no-implied-eval");
        Self {
            plus_operands: to_string | when(has("restrict-plus-operands"), 0),
            template_expressions: to_string
                | when(
                    has("restrict-template-expressions"),
                    detail::SYMBOL | detail::ELEMENTS | detail::BASES,
                ),
            first_argument: to_string | when(implied_eval, detail::SYMBOL | detail::CALLABLE),
            implied_eval_handler: implied_eval,
            stringified_receiver: to_string,
            function_callee: when(implied_eval, detail::SYMBOL),
            void_argument: when(has("no-meaningless-void-operator"), 0),
            spread_argument: when(
                has("no-misused-spread"),
                detail::SYMBOL
                    | detail::ELEMENTS
                    | detail::BASES
                    | detail::CALLABLE
                    | detail::OBJECT_KIND,
            ),
            members: has("unbound-method"),
        }
    }
}

/// Type tree details an enabled rule needs for a child position.
pub(super) fn wants_tree(needs: TreeNeeds, parent: &LintNode, key: &str, index: usize) -> u8 {
    match parent.kind.as_str() {
        "BinaryExpression"
            if parent.field_str("operator") == Some("+") && matches!(key, "left" | "right") =>
        {
            needs.plus_operands
        }
        "AssignmentExpression"
            if parent.field_str("operator") == Some("+=") && matches!(key, "left" | "right") =>
        {
            needs.plus_operands
        }
        "TemplateLiteral" if key == "expressions" => needs.template_expressions,
        "CallExpression" | "NewExpression" if key == "arguments" && index == 0 => {
            needs.first_argument
                | if needs.implied_eval_handler
                    && parent.child("callee").is_some_and(names_eval_like)
                {
                    detail::ROOT_CALLABLE
                } else {
                    0
                }
        }
        "CallExpression" | "NewExpression"
            if key == "callee" && parent.child("callee").is_some_and(names_function) =>
        {
            needs.function_callee
        }
        "UnaryExpression" if key == "argument" => needs.void_argument,
        "SpreadElement" | "JSXSpreadAttribute" if key == "argument" => needs.spread_argument,
        "MemberExpression"
            if key == "object"
                && parent
                    .child("property")
                    .and_then(|property| property.field_str("name"))
                    .is_some_and(|name| matches!(name, "toString" | "toLocaleString" | "join")) =>
        {
            needs.stringified_receiver
        }
        _ => 0,
    }
}

/// `Function` or `<global>.Function`, the constructor no-implied-eval checks.
fn names_function(callee: &LintNode) -> bool {
    match callee.kind.as_str() {
        "Identifier" => callee.field_str("name") == Some("Function"),
        "MemberExpression" => callee.child("property").is_some_and(|property| {
            property.field_str("name") == Some("Function")
                || property.field_str("value") == Some("Function")
        }),
        _ => false,
    }
}

/// An eval-like global name recognized by no-implied-eval.
fn names_eval_like(callee: &LintNode) -> bool {
    let name = match callee.kind.as_str() {
        "Identifier" => callee.field_str("name"),
        "MemberExpression"
            if callee.child("object").is_some_and(|object| {
                object.kind == "Identifier"
                    && object
                        .field_str("name")
                        .is_some_and(|name| matches!(name, "global" | "globalThis" | "window"))
            }) =>
        {
            let property = callee.child("property");
            match property.map(|property| property.kind.as_str()) {
                Some("Identifier") if callee.field_bool("computed") != Some(true) => {
                    property.and_then(|property| property.field_str("name"))
                }
                Some("Literal") => property.and_then(|property| property.field_str("value")),
                _ => None,
            }
        }
        _ => None,
    };
    name.is_some_and(|name| {
        matches!(
            name,
            "setImmediate" | "setInterval" | "setTimeout" | "execScript"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{TreeNeeds, wants_tree};
    use crate::linter::corsa_session::detail;
    use corsa::lint::LintNode;
    use serde_json::{Value, json};

    fn identifier(name: &str) -> Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 10 },
            "fields": { "name": name }
        })
    }

    fn call(callee: Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 20 },
            "children": { "callee": callee }
        }))
        .expect("call expression")
    }

    #[test]
    fn scopes_root_callable_to_eval_like_handlers() {
        let needs = TreeNeeds::from_rules(&["no-implied-eval"]);
        let inherited = detail::STRUCTURE | detail::SYMBOL | detail::CALLABLE;

        assert_eq!(
            wants_tree(needs, &call(identifier("run")), "arguments", 0),
            inherited
        );
        assert_eq!(
            wants_tree(needs, &call(identifier("setTimeout")), "arguments", 0),
            inherited | detail::ROOT_CALLABLE
        );

        let global_timer = json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 20 },
            "fields": { "computed": true },
            "children": {
                "object": identifier("globalThis"),
                "property": {
                    "kind": "Literal",
                    "range": { "start": 11, "end": 19 },
                    "fields": { "value": "setInterval" }
                }
            }
        });
        assert_eq!(
            wants_tree(needs, &call(global_timer), "arguments", 0),
            inherited | detail::ROOT_CALLABLE
        );
    }
}

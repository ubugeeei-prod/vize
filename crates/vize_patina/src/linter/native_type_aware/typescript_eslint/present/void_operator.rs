//! Rule-local compatibility for `no-meaningless-void-operator` operands.
use corsa::lint::LintNode;
use serde_json::Value;

/// Expose the checker's `isThenableType` verdict on a `void` operand.
///
/// typescript-eslint leaves `void promise` alone so the rule does not fight
/// no-floating-promises, and asks `tsutils.isThenableType` whether the operand
/// is a promise. `corsa_core` 2.0.0-beta.5 approximates that question from the
/// rendered type text: any text that starts with `Promise` counts, so
/// `Promise<unknown>[]` (an array, never awaited by `void`) is let through.
///
/// The rule receives a clone. When the checker answered, a thenable operand
/// gets a `then` member and a non-thenable one gets a type text that names no
/// promise, so corsa's approximation agrees with the checker on this rule's
/// input only. Without a checker answer the operand is left untouched.
pub(super) fn expose_thenable_operand(node: &mut LintNode) {
    if node.kind != "UnaryExpression" || node.field_str("operator") != Some("void") {
        return;
    }
    let Some(argument) = node.children.get_mut("argument") else {
        return;
    };
    // A discarded call is judged on its return type, never on thenability.
    if discarded(argument).kind == "CallExpression" {
        return;
    }
    let thenable = argument
        .fields
        .get("__promiseFacts")
        .and_then(|facts| facts.get("thenable"))
        .and_then(Value::as_bool);
    match thenable {
        Some(true) => {
            if !argument.property_names.iter().any(|name| name == "then") {
                argument.property_names.push("then".into());
            }
        }
        Some(false) if !argument.type_texts.is_empty() => {
            argument.type_texts = vec![NON_THENABLE.into()];
        }
        _ => {}
    }
}

/// typescript-eslint's `unwrapVoidArgument`: the expression `void` discards.
fn discarded(node: &LintNode) -> &LintNode {
    let mut current = node;
    loop {
        let next = match current.kind.as_str() {
            "ChainExpression"
            | "TSAsExpression"
            | "TSNonNullExpression"
            | "TSSatisfiesExpression"
            | "TSTypeAssertion"
            | "ParenthesizedExpression" => current.child("expression"),
            "SequenceExpression" => current
                .child_list("expressions")
                .and_then(|expressions| expressions.last()),
            _ => None,
        };
        match next {
            Some(next) => current = next,
            None => return current,
        }
    }
}

/// The type text a checker-proven non-thenable operand is presented with.
/// The rule reads the operand's text only to tell a promise apart; the
/// message names no type.
const NON_THENABLE: &str = "object";

#[cfg(test)]
mod tests {
    use super::expose_thenable_operand;
    use corsa::lint::LintNode;
    use serde_json::json;

    fn void_of(texts: &[&str], thenable: Option<bool>) -> LintNode {
        let mut fields = json!({});
        if let Some(thenable) = thenable {
            fields = json!({ "__promiseFacts": { "thenable": thenable } });
        }
        serde_json::from_value(json!({
            "kind": "UnaryExpression",
            "range": { "start": 0, "end": 17 },
            "fields": { "operator": "void" },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 5, "end": 17 },
                    "typeTexts": texts,
                    "fields": fields,
                }
            }
        }))
        .expect("lint node")
    }

    fn argument(node: &LintNode) -> (&[String], &[String]) {
        let argument = node.child("argument").expect("argument");
        (&argument.type_texts, &argument.property_names)
    }

    #[test]
    fn promise_arrays_lose_the_promise_text() {
        let mut node = void_of(&["Promise<unknown>[]"], Some(false));
        expose_thenable_operand(&mut node);
        assert_eq!(argument(&node).0, ["object"]);
    }

    #[test]
    fn checker_thenables_gain_a_then_member() {
        let mut node = void_of(&["Custom"], Some(true));
        expose_thenable_operand(&mut node);
        assert_eq!(argument(&node).1, ["then"]);
    }

    #[test]
    fn discarded_calls_keep_their_return_texts() {
        let mut node = void_of(&["void"], Some(false));
        node.children.get_mut("argument").expect("argument").kind = "CallExpression".into();
        expose_thenable_operand(&mut node);
        assert_eq!(argument(&node).0, ["void"]);
    }

    #[test]
    fn unanswered_and_untyped_operands_stay_untouched() {
        let mut node = void_of(&["Promise<unknown>[]"], None);
        expose_thenable_operand(&mut node);
        assert_eq!(argument(&node).0, ["Promise<unknown>[]"]);
        let mut node = void_of(&[], Some(false));
        expose_thenable_operand(&mut node);
        assert!(argument(&node).0.is_empty());
    }
}

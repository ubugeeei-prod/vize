//! `no-misused-spread` input aligned with the native runtime's syntax kinds.
use corsa::lint::LintNode;
use serde_json::Value;

// TypeScript 7 SyntaxKind values (microsoft/typescript-go internal/ast/kind.go)
// of the two class declarations, as the native checker reports them.
const CLASS_DECLARATION: u64 = 264;
const CLASS_EXPRESSION: u64 = 232;
// The values corsa_core 2.0.0-beta.5's `isClassDeclaration` compares with.
// Neither can be the value declaration of a symbol on TypeScript 7 (they are
// `InterfaceDeclaration` and `OmittedExpression` there), so only class
// declarations ever carry them after the translation below.
const CORSA_CLASS_DECLARATION: u64 = 265;
const CORSA_CLASS_EXPRESSION: u64 = 233;

/// Compatibility boundary for corsa_core 2.0.0-beta.5, whose
/// `noClassDeclarationSpreadInObject` check reads class value declarations as
/// kinds 265/233 while the native runtime reports ClassDeclaration as 264 and
/// ClassExpression as 232, so `{ ...SomeClass }` never matched. Vize keeps the
/// shared type tree exact; only this rule's cloned spread argument presents
/// the kinds the dependency compares with. Remove after the dependency reads
/// the runtime's kinds.
pub(super) fn expose_class_declarations(node: &mut LintNode) {
    if !matches!(node.kind.as_str(), "SpreadElement" | "JSXSpreadAttribute") {
        return;
    }
    let Some(tree) = node
        .children
        .get_mut("argument")
        .and_then(|argument| argument.fields.get_mut("__typeTree"))
    else {
        return;
    };
    translate(tree);
}

fn translate(tree: &mut Value) {
    let Some(object) = tree.as_object_mut() else {
        return;
    };
    if let Some(kind) = object.get_mut("valueDeclarationKind") {
        let translated = match kind.as_u64() {
            Some(CLASS_DECLARATION) => Some(CORSA_CLASS_DECLARATION),
            Some(CLASS_EXPRESSION) => Some(CORSA_CLASS_EXPRESSION),
            _ => None,
        };
        if let Some(translated) = translated {
            *kind = Value::from(translated);
        }
    }
    // The rule recurses through union and intersection constituents only.
    if let Some(parts) = object.get_mut("types").and_then(Value::as_array_mut) {
        for part in parts {
            translate(part);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::expose_class_declarations;
    use corsa::lint::LintNode;
    use serde_json::{Value, json};

    fn spread(tree: Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "SpreadElement",
            "range": { "start": 0, "end": 4 },
            "fields": { "__parentKind": "ObjectExpression" },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 3, "end": 4 },
                    "fields": { "name": "A", "__typeTree": tree }
                }
            }
        }))
        .expect("spread node")
    }

    fn kinds(node: &LintNode) -> Value {
        let tree = &node.child("argument").expect("argument").fields["__typeTree"];
        json!([
            tree["types"][0]["valueDeclarationKind"],
            tree["types"][1]["valueDeclarationKind"],
            tree["types"][2]["valueDeclarationKind"],
        ])
    }

    #[test]
    fn class_value_declarations_reach_the_rule_kinds() {
        let mut node = spread(json!({
            "flags": 1 << 27,
            "types": [
                { "flags": 1 << 20, "valueDeclarationKind": 264 },
                { "flags": 1 << 20, "valueDeclarationKind": 232 },
                { "flags": 1 << 20, "valueDeclarationKind": 263 },
            ],
        }));
        expose_class_declarations(&mut node);
        assert_eq!(kinds(&node), json!([265, 233, 263]));
        // Idempotent: an already translated tree is unchanged.
        expose_class_declarations(&mut node);
        assert_eq!(kinds(&node), json!([265, 233, 263]));
    }
}

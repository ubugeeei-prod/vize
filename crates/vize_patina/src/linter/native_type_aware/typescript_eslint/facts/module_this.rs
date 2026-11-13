//! `this` outside every function and class.
use corsa::lint::LintNode;
use serde_json::{Value, json};

/// TypeScript 7 `TypeFlags.Undefined`.
const UNDEFINED: u32 = 1 << 2;

/// Kinds that give `this` a binding of its own.
const THIS_SCOPES: &[&str] = &[
    "FunctionDeclaration",
    "FunctionExpression",
    "ClassBody",
    "TSModuleBlock",
];

/// A module's top-level `this` is `undefined`. The projection moves a plain
/// `<script>` body into a setup function, where the checker types `this` as
/// `any`; restore the module semantics for `this` outside every function and
/// class.
pub(super) fn stamp_module_this(node: &mut LintNode) {
    let bound = node
        .fields
        .get("__ancestorFacts")
        .and_then(Value::as_array)
        .is_some_and(|ancestors| {
            ancestors.iter().any(|ancestor| {
                ancestor
                    .get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| THIS_SCOPES.contains(&kind))
            })
        });
    if !bound {
        mark_module_this(node);
    }
}

fn mark_module_this(node: &mut LintNode) {
    if THIS_SCOPES.contains(&node.kind.as_str()) {
        return;
    }
    if node.kind == "ThisExpression" {
        node.type_texts = vec!["undefined".into()];
        node.property_names.clear();
        node.fields
            .insert("__type".into(), json!({ "id": "", "flags": UNDEFINED }));
        return;
    }
    for child in node.children.values_mut() {
        mark_module_this(child);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            mark_module_this(child);
        }
    }
}

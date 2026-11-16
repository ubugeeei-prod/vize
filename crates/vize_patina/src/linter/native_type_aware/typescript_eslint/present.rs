//! Report ranges aligned with typescript-eslint.
mod require_await;
mod void_operator;

use std::collections::BTreeMap;

use corsa::lint::{LintNode, TextRange};
use serde_json::Value;
use vize_l0::{String, cstr};

pub(super) fn prepare(rule: &str, node: &LintNode, configured: Option<&Value>) -> LintNode {
    let mut owned = node.clone();
    match rule {
        "no-unsafe-return" => expose_awaited_any(&mut owned),
        "no-base-to-string" => {
            constrain_stringified_types(&mut owned);
            supply_tostring_argument(&mut owned);
        }
        "require-await" => require_await::expose_non_thenable_any_returns(&mut owned),
        "no-meaningless-void-operator" => void_operator::expose_thenable_operand(&mut owned),
        "restrict-plus-operands" if disallows_nullish(configured) => {
            expose_nullish_union_flags(&mut owned);
        }
        _ => {}
    }
    owned
}

/// Corsa's current `no-unsafe-return` reader can unwrap a top-level
/// `Promise<T>`, while the checker can also identify thenables hidden behind
/// unions, intersections and inherited aliases. Present that already-proven
/// fact as a Promise reference on this rule's private clone. The original
/// checker identity and constituents stay intact for explicit-return and
/// assignment comparisons.
fn expose_awaited_any(node: &mut LintNode) {
    let returned = match node.kind.as_str() {
        "ReturnStatement" => node.children.get_mut("argument"),
        "ArrowFunctionExpression" => node.children.get_mut("body"),
        _ => None,
    };
    let Some(returned) = returned else {
        return;
    };
    let Some(awaited) = returned.fields.get("__unsafeReturnAwaitedAny").cloned() else {
        return;
    };
    let Some(detail) = returned
        .fields
        .get_mut("__typeDetail")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    let object_flags = detail
        .get("objectFlags")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    detail.insert("objectFlags".into(), Value::from(object_flags | 4));
    detail.insert("text".into(), Value::from("Promise<any>"));
    detail.insert("typeArguments".into(), Value::Array(vec![awaited]));
    detail.insert("isArray".into(), Value::Bool(false));
    detail.insert("isTuple".into(), Value::Bool(false));
}

fn disallows_nullish(configured: Option<&Value>) -> bool {
    configured
        .and_then(|options| options.get(0))
        .and_then(|options| options.get("allowNullish"))
        .and_then(Value::as_bool)
        == Some(false)
}

// Compatibility boundary for corsa_core 2.0.0-beta.5. Its
// restrict-plus-operands implementation checks null and undefined on the
// union's root flags while checking every other option across union parts.
// Vize keeps the checker tree shared and exact; only this rule's cloned input
// exposes those two constituent flags at each union root. The transform is
// idempotent, so a runtime that already exposes them is unchanged. Remove this
// after the dependency checks nullish flags with its union-parts helper.
fn expose_nullish_union_flags(node: &mut LintNode) {
    if !matches!(
        node.kind.as_str(),
        "BinaryExpression" | "AssignmentExpression"
    ) {
        return;
    }
    for key in ["left", "right"] {
        let Some(tree) = node
            .children
            .get_mut(key)
            .and_then(|operand| operand.fields.get_mut("__typeTree"))
        else {
            continue;
        };
        normalize_nullish_unions(tree);
    }
}

const TYPE_FLAG_UNDEFINED: u64 = 1 << 2;
const TYPE_FLAG_NULL: u64 = 1 << 3;
const TYPE_FLAG_UNION: u64 = 1 << 27;
const TYPE_FLAG_NULLISH: u64 = TYPE_FLAG_UNDEFINED | TYPE_FLAG_NULL;

/// Normalize every union in a type tree without leaking flags through an
/// intersection or a type-parameter boundary.
fn normalize_nullish_unions(tree: &mut Value) -> Option<u64> {
    let object = tree.as_object_mut()?;
    let mut flags = object.get("flags")?.as_u64()?;

    for key in ["constraint", "arrayElement"] {
        if let Some(child) = object.get_mut(key) {
            normalize_nullish_unions(child);
        }
    }
    for key in ["types", "tupleElements"] {
        if let Some(children) = object.get_mut(key).and_then(Value::as_array_mut) {
            for child in children {
                normalize_nullish_unions(child);
            }
        }
    }

    if flags & TYPE_FLAG_UNION != 0 {
        let constituent_nullish = object
            .get("types")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|part| part.get("flags").and_then(Value::as_u64))
            .fold(0, |found, part| found | (part & TYPE_FLAG_NULLISH));
        let normalized = flags | constituent_nullish;
        if normalized != flags {
            flags = normalized;
            object.insert("flags".into(), Value::from(flags));
        }
    }
    Some(flags)
}

/// Mirror `getConstrainedTypeAtLocation` for the exact expression shapes this
/// rule stringifies. Corsa currently follows type-parameter constraints but
/// not indexed-access constraints such as `Record<K, V>[K]` (#13004).
fn constrain_stringified_types(node: &mut LintNode) {
    let constrain = |node: &mut LintNode| {
        if let Some(tree) = node.fields.get_mut("__typeTree") {
            constrain_tree(tree);
        }
    };
    match node.kind.as_str() {
        "BinaryExpression" | "AssignmentExpression" => {
            for key in ["left", "right"] {
                if let Some(expression) = node.children.get_mut(key) {
                    constrain(expression);
                }
            }
        }
        "CallExpression" => {
            if let Some(argument) = node
                .child_lists
                .get_mut("arguments")
                .and_then(|arguments| arguments.first_mut())
            {
                constrain(argument);
            }
            if let Some(object) = node
                .children
                .get_mut("callee")
                .and_then(|callee| callee.children.get_mut("object"))
            {
                constrain(object);
            }
        }
        "TemplateLiteral" => {
            if let Some(expressions) = node.child_lists.get_mut("expressions") {
                for expression in expressions {
                    constrain(expression);
                }
            }
        }
        _ => {}
    }
}

fn constrain_tree(tree: &mut Value) {
    let Some(object) = tree.as_object_mut() else {
        return;
    };
    if let Some(mut constraint) = object.remove("constraint") {
        constrain_tree(&mut constraint);
        *tree = constraint;
        return;
    }
    if let Some(child) = object.get_mut("arrayElement") {
        constrain_tree(child);
    }
    for key in ["types", "tupleElements"] {
        if let Some(children) = object.get_mut(key).and_then(Value::as_array_mut) {
            for child in children {
                constrain_tree(child);
            }
        }
    }
}

fn supply_tostring_argument(node: &mut LintNode) {
    if node.kind != "CallExpression" {
        return;
    }
    if node
        .child_list("arguments")
        .is_some_and(|arguments| !arguments.is_empty())
    {
        return;
    }
    let property = node
        .child("callee")
        .and_then(|callee| callee.child("property"))
        .and_then(|property| property.field_str("name"));
    if !matches!(property, Some("toString" | "toLocaleString")) {
        return;
    }
    node.child_lists
        .entry("arguments".into())
        .or_default()
        .push(LintNode {
            kind: "Literal".into(),
            range: TextRange::new(node.range.end, node.range.end),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        });
}

pub(super) fn range(rule: &str, node: &LintNode, start: u32, end: u32) -> (u32, u32) {
    match rule {
        "no-unsafe-return" => unsafe_return(node, start, end),
        _ => (start, end),
    }
}

pub(super) fn message(rule: &str, node: &LintNode, start: u32, fallback: String) -> String {
    if rule != "no-unnecessary-type-parameters" {
        return fallback;
    }
    let Some(param) = node.child_list("params").and_then(|params| {
        params
            .iter()
            .find(|param| param.range.start <= start && start < param.range.end)
    }) else {
        return fallback;
    };
    let name = param.field_str("__typeParameterName").unwrap_or("T");
    let uses = match param.field_f64("__typeParameterUsageCount") {
        Some(1.0) => "never used",
        Some(2.0) => "used only once",
        _ => return fallback,
    };
    let descriptor = match node.field_str("__ownerKind") {
        Some("ClassDeclaration" | "ClassExpression") => "class",
        _ => "function",
    };
    cstr!("Type parameter {name} is {uses} in the {descriptor} signature.")
}

fn unsafe_return(node: &LintNode, start: u32, end: u32) -> (u32, u32) {
    if node.kind == "ReturnStatement" {
        (node.range.start, node.range.end.max(end))
    } else {
        (start, end)
    }
}

#[cfg(test)]
mod tests {
    use corsa::lint::{LintNode, TextRange, run_default_type_aware_rule_owned};
    use serde_json::{Value, json};

    use super::{
        TYPE_FLAG_NULL, TYPE_FLAG_UNDEFINED, TYPE_FLAG_UNION, constrain_tree,
        normalize_nullish_unions, prepare,
    };

    #[test]
    fn constrained_indexed_accesses_are_normalized_inside_arrays() {
        let mut tree = json!({
            "flags": 1_048_576,
            "arrayElement": {
                "flags": 33_554_432,
                "text": "Record<K, Badge>[K]",
                "constraint": {
                    "flags": 1_048_576,
                    "text": "Badge",
                    "toString": true
                }
            }
        });
        constrain_tree(&mut tree);
        assert_eq!(
            tree,
            json!({
                "flags": 1_048_576,
                "arrayElement": {
                    "flags": 1_048_576,
                    "text": "Badge",
                    "toString": true
                }
            })
        );
    }

    fn operand(start: u32, end: u32, tree: Value) -> Value {
        json!({
            "kind": "Identifier",
            "range": { "start": start, "end": end },
            "fields": { "__typeTree": tree }
        })
    }

    fn plus(left: Value, right: Value, allow_nullish: bool) -> LintNode {
        serde_json::from_value(json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 12 },
            "fields": {
                "operator": "+",
                "__ruleOptions": [{
                    "allowAny": false,
                    "allowBoolean": false,
                    "allowNullish": allow_nullish,
                    "allowNumberAndString": true,
                    "allowRegExp": false,
                    "skipCompoundAssignments": false
                }]
            },
            "children": { "left": left, "right": right }
        }))
        .expect("plus expression")
    }

    fn string() -> Value {
        json!({ "id": "string", "flags": 1 << 5, "text": "string" })
    }

    fn string_or_undefined() -> Value {
        json!({
            "id": "union",
            "flags": TYPE_FLAG_UNION,
            "text": "string | undefined",
            "types": [
                string(),
                {
                    "id": "undefined",
                    "flags": TYPE_FLAG_UNDEFINED,
                    "text": "undefined"
                }
            ]
        })
    }

    #[test]
    fn restrict_plus_exposes_nullish_union_only_to_its_owned_rule_input() {
        let original = plus(
            operand(0, 5, string_or_undefined()),
            operand(8, 12, string()),
            false,
        );
        let prepared = prepare(
            "restrict-plus-operands",
            &original,
            original.fields.get("__ruleOptions"),
        );
        let original_flags = original
            .child("left")
            .and_then(|left| left.fields.get("__typeTree"))
            .and_then(|tree| tree.get("flags"))
            .and_then(Value::as_u64);
        let prepared_flags = prepared
            .child("left")
            .and_then(|left| left.fields.get("__typeTree"))
            .and_then(|tree| tree.get("flags"))
            .and_then(Value::as_u64);
        assert_eq!(original_flags, Some(TYPE_FLAG_UNION));
        assert_eq!(prepared_flags, Some(TYPE_FLAG_UNION | TYPE_FLAG_UNDEFINED));

        let diagnostics = run_default_type_aware_rule_owned("restrict-plus-operands", prepared)
            .expect("registered rule");
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.rule_name, "restrict-plus-operands");
        assert_eq!(diagnostic.message_id, "invalid");
        assert_eq!(
            diagnostic.message,
            "Invalid operand for a '+' operation. Operands must each be a number or string. Got `string | undefined`."
        );
        assert!(diagnostic.data.is_empty());
        assert_eq!(diagnostic.range, TextRange::new(0, 5));
        assert!(diagnostic.suggestions.is_empty());
    }

    #[test]
    fn restrict_plus_preserves_allow_nullish_true() {
        let prepared = prepare(
            "restrict-plus-operands",
            &plus(
                operand(0, 5, string_or_undefined()),
                operand(8, 12, string()),
                true,
            ),
            Some(&json!([{ "allowNullish": true }])),
        );
        assert!(
            run_default_type_aware_rule_owned("restrict-plus-operands", prepared)
                .expect("registered rule")
                .is_empty()
        );
    }

    #[test]
    fn nullish_normalization_respects_nested_type_boundaries_and_is_idempotent() {
        let mut tree = json!({
            "id": "T",
            "flags": 1 << 19,
            "text": "T",
            "constraint": {
                "id": "nested-union",
                "flags": TYPE_FLAG_UNION,
                "text": "string | null",
                "types": [
                    string(),
                    {
                        "id": "null",
                        "flags": TYPE_FLAG_NULL,
                        "text": "null"
                    }
                ]
            },
            "types": [{
                "id": "intersection",
                "flags": 1 << 28,
                "text": "Brand & (string | undefined)",
                "types": [{
                    "id": "inner-union",
                    "flags": TYPE_FLAG_UNION,
                    "text": "string | undefined",
                    "types": [
                        string(),
                        {
                            "id": "undefined",
                            "flags": TYPE_FLAG_UNDEFINED,
                            "text": "undefined"
                        }
                    ]
                }]
            }]
        });
        let first = normalize_nullish_unions(&mut tree);
        let once = tree.clone();
        let second = normalize_nullish_unions(&mut tree);

        assert_eq!(first, Some(1 << 19));
        assert_eq!(second, Some(1 << 19));
        assert_eq!(tree, once, "normalization must be idempotent");
        assert_eq!(tree["flags"], json!(1 << 19));
        assert_eq!(
            tree["constraint"]["flags"],
            json!(TYPE_FLAG_UNION | TYPE_FLAG_NULL)
        );
        assert_eq!(tree["types"][0]["flags"], json!(1 << 28));
        assert_eq!(
            tree["types"][0]["types"][0]["flags"],
            json!(TYPE_FLAG_UNION | TYPE_FLAG_UNDEFINED)
        );
    }
}

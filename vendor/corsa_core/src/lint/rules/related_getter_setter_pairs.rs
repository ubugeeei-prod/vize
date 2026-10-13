use std::collections::BTreeMap;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::lint::helpers::{child_list, identifier_name};
use crate::utils::split_top_level_type_text;

/// Type-aware rule that requires a `get()`'s type to be assignable to the
/// matching `set()`'s parameter type on the same container.
///
/// Faithful port of the tsgolint builtin rule
/// `related-getter-setter-pairs`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RelatedGetterSetterPairsRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "mismatch",
    description: "`get()` type should be assignable to its equivalent `set()` type.",
}];

// Mirrors the Go listeners:
//   ast.KindClassDeclaration, ast.KindClassExpression,
//   ast.KindInterfaceDeclaration, ast.KindTypeLiteral
// expressed using TS-ESTree node kinds.
const LISTENERS: &[&str] = &[
    "ClassDeclaration",
    "ClassExpression",
    "TSInterfaceDeclaration",
    "TSTypeLiteral",
];

impl RustLintRule for RelatedGetterSetterPairsRule {
    fn name(&self) -> &'static str {
        "related-getter-setter-pairs"
    }

    fn docs_description(&self) -> &'static str {
        "Require paired getters and setters to use compatible types."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        check_members(ctx, node);
    }
}

/// Visited accessor with the facts needed to evaluate the pair.
struct GetAccessor {
    /// Type texts rendered for the getter's return type.
    return_type_texts: Vec<String>,
    /// Range of the getter's return type annotation node (reported on mismatch).
    type_range: TextRange,
    /// Raw checker result, when the host computed assignability for this getter.
    assignable_to_setter: Option<bool>,
}

struct SetAccessor {
    /// Type texts rendered for the setter's single parameter.
    param_type_texts: Vec<String>,
}

fn check_members(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let members = members_of(node);
    if members.is_empty() {
        return;
    }

    // Replicate the Go single-pass pairing: as soon as a getter sees its
    // matching setter (or vice versa) the pair is evaluated. We collect first,
    // then evaluate, which is equivalent because at most one get and one set
    // per name participate and a mismatch is reported on the getter.
    let mut getters: BTreeMap<String, GetAccessor> = BTreeMap::new();
    let mut setters: BTreeMap<String, SetAccessor> = BTreeMap::new();

    for member in members {
        match member.field_str("kind") {
            Some("get") => {
                // Go: `if m.Type == nil { continue }` — skip getters with no
                // explicit return type annotation.
                let Some((type_range, return_type_texts)) = getter_return_type(member) else {
                    continue;
                };
                let Some(name) = member_name(member) else {
                    continue;
                };
                getters.entry(name).or_insert(GetAccessor {
                    return_type_texts,
                    type_range,
                    assignable_to_setter: member.field_bool("__getterTypeAssignableToSetter"),
                });
            }
            Some("set") => {
                let params = setter_params(member);
                // Go: `if len(m.Parameters.Nodes) != 1 { continue }`.
                if params.len() != 1 {
                    continue;
                }
                let Some(name) = member_name(member) else {
                    continue;
                };
                setters.entry(name).or_insert(SetAccessor {
                    param_type_texts: params[0].type_texts.clone(),
                });
            }
            _ => {}
        }
    }

    for (name, getter) in &getters {
        let Some(setter) = setters.get(name) else {
            continue;
        };
        if !is_assignable(getter, setter) {
            ctx.report("mismatch", getter.type_range);
        }
    }
}

/// Returns the container's member list across class / interface / type-literal
/// shapes.
fn members_of(node: &LintNode) -> &[LintNode] {
    // ClassDeclaration / ClassExpression -> body: ClassBody { body: [...] }
    if let Some(body) = node.child("body") {
        if let Some(members) = body.child_list("body") {
            return members;
        }
        // TSInterfaceDeclaration -> body: TSInterfaceBody { body: [...] }
        if let Some(members) = body.child_list("members") {
            return members;
        }
    }
    // TSTypeLiteral -> members: [...]
    if let Some(members) = node.child_list("members") {
        return members;
    }
    // Fallback: some hosts inline the class body members directly.
    child_list(node, "body")
}

/// Resolves the getter's return type annotation range and rendered type texts.
///
/// Returns `None` when the getter has no explicit return type (matches the Go
/// `m.Type == nil` skip).
fn getter_return_type(member: &LintNode) -> Option<(TextRange, Vec<String>)> {
    // Class accessors carry the signature on `value` (a FunctionExpression /
    // TSEmptyBodyFunctionExpression); interface / type-literal method
    // signatures carry it directly on the member.
    let signature = member.child("value").unwrap_or(member);

    let return_type = signature
        .child("returnType")
        .or_else(|| member.child("returnType"))?;

    // `returnType` is a TSTypeAnnotation; Go reports the inner type node
    // (`getter.Type`), i.e. the annotation without the leading colon.
    let type_node = return_type.child("typeAnnotation").unwrap_or(return_type);

    let texts = collect_return_type_texts(member, signature, type_node);
    Some((type_node.range, texts))
}

/// Gathers the getter return type texts, preferring host-rendered facts and
/// falling back to the annotation node's own type texts / source text.
fn collect_return_type_texts(
    member: &LintNode,
    signature: &LintNode,
    type_node: &LintNode,
) -> Vec<String> {
    let from_member = string_array_field(member, "__returnTypeTexts");
    if !from_member.is_empty() {
        return from_member;
    }
    let from_signature = string_array_field(signature, "__returnTypeTexts");
    if !from_signature.is_empty() {
        return from_signature;
    }
    if !type_node.type_texts.is_empty() {
        return type_node.type_texts.clone();
    }
    type_node.text.iter().cloned().collect()
}

/// Returns the setter's parameter nodes across the supported shapes.
fn setter_params(member: &LintNode) -> &[LintNode] {
    let signature = member.child("value").unwrap_or(member);
    if let Some(params) = signature.child_list("params") {
        return params;
    }
    if let Some(params) = member.child_list("params") {
        return params;
    }
    &[]
}

/// Resolves the accessor name from its key (Go: `utils.GetNameFromMember`).
fn member_name(member: &LintNode) -> Option<String> {
    let key = member.child("key")?;
    identifier_name(key)
        .or_else(|| key.field_stringish("name"))
        .or_else(|| literal_key_value(key))
}

fn literal_key_value(key: &LintNode) -> Option<String> {
    if key.kind == "Literal" {
        return key.field_stringish("value");
    }
    None
}

/// Decides whether the getter type is assignable to the setter parameter type.
///
/// Prefers the raw `__getterTypeAssignableToSetter` checker fact when the host
/// provided it; otherwise falls back to a conservative textual union-subset
/// approximation over the rendered type texts.
fn is_assignable(getter: &GetAccessor, setter: &SetAccessor) -> bool {
    if let Some(assignable) = getter.assignable_to_setter {
        return assignable;
    }
    // When either side has no rendered type, do not report (cannot prove a
    // mismatch); this also mirrors the Go behaviour of only acting once both
    // types resolve.
    if getter.return_type_texts.is_empty() || setter.param_type_texts.is_empty() {
        return true;
    }
    textual_assignable(&getter.return_type_texts, &setter.param_type_texts)
}

/// Conservative assignability approximation: every top-level constituent of the
/// getter's union must appear among the setter parameter's union constituents.
fn textual_assignable(getter_texts: &[String], setter_texts: &[String]) -> bool {
    let setter_parts = union_parts(setter_texts);
    union_parts(getter_texts)
        .into_iter()
        .all(|part| setter_parts.iter().any(|candidate| candidate == &part))
}

fn union_parts(texts: &[String]) -> Vec<String> {
    let mut parts = Vec::new();
    for text in texts {
        for part in split_top_level_type_text(text, '|') {
            let normalized = part.trim().to_owned();
            if !normalized.is_empty() && !parts.contains(&normalized) {
                parts.push(normalized);
            }
        }
    }
    parts
}

fn string_array_field(node: &LintNode, key: &str) -> Vec<String> {
    node.fields
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::RelatedGetterSetterPairsRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = RelatedGetterSetterPairsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    /// Builds an interface-style member list container.
    fn interface_with(members: serde_json::Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "TSInterfaceDeclaration",
            "range": { "start": 0, "end": 100 },
            "children": {
                "body": {
                    "kind": "TSInterfaceBody",
                    "range": { "start": 10, "end": 90 },
                    "childLists": { "body": members }
                }
            }
        }))
        .unwrap()
    }

    fn getter(
        name: &str,
        return_texts: &[&str],
        type_start: u32,
        type_end: u32,
    ) -> serde_json::Value {
        json!({
            "kind": "TSMethodSignature",
            "range": { "start": type_start, "end": type_end },
            "fields": { "kind": "get", "__returnTypeTexts": return_texts },
            "children": {
                "key": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": name } },
                "returnType": {
                    "kind": "TSTypeAnnotation",
                    "range": { "start": type_start, "end": type_end },
                    "children": {
                        "typeAnnotation": {
                            "kind": "TSTypeReference",
                            "range": { "start": type_start, "end": type_end },
                            "typeTexts": return_texts
                        }
                    }
                }
            },
            "childLists": { "params": [] }
        })
    }

    fn setter(name: &str, param_texts: &[&str]) -> serde_json::Value {
        json!({
            "kind": "TSMethodSignature",
            "range": { "start": 0, "end": 1 },
            "fields": { "kind": "set" },
            "children": {
                "key": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": name } }
            },
            "childLists": {
                "params": [
                    { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "typeTexts": param_texts }
                ]
            }
        })
    }

    #[test]
    fn reports_when_getter_wider_than_setter() {
        // get value(): string | undefined; set value(newValue: string);
        let node = interface_with(json!([
            getter("value", &["string | undefined"], 16, 34),
            setter("value", &["string"]),
        ]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "mismatch");
        assert_eq!(diagnostics[0].range.start, 16);
        assert_eq!(diagnostics[0].range.end, 34);
    }

    #[test]
    fn reports_when_unrelated_types() {
        // get value(): number; set value(newValue: string);
        let node = interface_with(json!([
            getter("value", &["number"], 16, 22),
            setter("value", &["string"]),
        ]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "mismatch");
        assert_eq!(diagnostics[0].range.start, 16);
        assert_eq!(diagnostics[0].range.end, 22);
    }

    #[test]
    fn allows_getter_narrower_than_setter() {
        // get value(): string; set value(newValue: string | undefined);
        let node = interface_with(json!([
            getter("value", &["string"], 16, 22),
            setter("value", &["string | undefined"]),
        ]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn allows_matching_types() {
        // get value(): number; set value(newValue: number);
        let node = interface_with(json!([
            getter("value", &["number"], 16, 22),
            setter("value", &["number"]),
        ]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_setter_with_no_params() {
        // get value(): number; set value();  -> setter param count != 1, skip.
        let node = interface_with(json!([
            getter("value", &["number"], 16, 22),
            {
                "kind": "TSMethodSignature",
                "range": { "start": 0, "end": 1 },
                "fields": { "kind": "set" },
                "children": {
                    "key": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "value" } }
                },
                "childLists": { "params": [] }
            },
        ]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_getter_without_return_type() {
        // get value() {...} with no return type annotation -> skip.
        let node = interface_with(json!([
            {
                "kind": "TSMethodSignature",
                "range": { "start": 0, "end": 1 },
                "fields": { "kind": "get" },
                "children": {
                    "key": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "value" } }
                },
                "childLists": { "params": [] }
            },
            setter("value", &["string"]),
        ]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn prefers_raw_assignability_fact() {
        // Identical type texts, but the host says it is NOT assignable.
        let mut getter_member = getter("a", &["GetType"], 12, 19);
        getter_member["fields"]["__getterTypeAssignableToSetter"] = json!(false);
        let node = interface_with(json!([getter_member, setter("a", &["GetType"])]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start, 12);
        assert_eq!(diagnostics[0].range.end, 19);
    }
}

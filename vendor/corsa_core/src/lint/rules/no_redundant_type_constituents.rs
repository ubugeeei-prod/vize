use std::collections::BTreeMap;

use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};

/// Type-aware rule that flags union/intersection constituents made redundant by
/// other constituents (top/bottom types and primitive-over-literal overrides).
///
/// Faithful port of the tsgolint Go rule `no-redundant-type-constituents`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoRedundantTypeConstituentsRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "errorTypeOverrides",
        description: "Constituent is an 'error' type that acts as 'any' and overrides all other types.",
    },
    RuleMessage {
        id: "literalOverridden",
        description: "Literal constituent is overridden by a primitive in this union type.",
    },
    RuleMessage {
        id: "overridden",
        description: "Constituent is overridden by other types in this type.",
    },
    RuleMessage {
        id: "overrides",
        description: "Constituent overrides all other types in this type.",
    },
    RuleMessage {
        id: "primitiveOverridden",
        description: "Primitive constituent is overridden by literals in this intersection type.",
    },
];

const LISTENERS: &[&str] = &["TSUnionType", "TSIntersectionType"];

impl RustLintRule for NoRedundantTypeConstituentsRule {
    fn name(&self) -> &'static str {
        "no-redundant-type-constituents"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow members of unions and intersections that do nothing or override type information."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "TSIntersectionType" => check_intersection(ctx, node),
            "TSUnionType" => check_union(ctx, node),
            _ => {}
        }
    }
}

/// Type-flag classification of a single resolved type part, mirroring the subset
/// of `checker.TypeFlags` the Go rule branches on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PartFlag {
    Any,
    /// An `error` type that acts as `any` (e.g. unresolved type reference).
    Error,
    Never,
    Unknown,
    BigInt,
    Boolean,
    Number,
    String,
    BigIntLiteral,
    BooleanLiteral,
    NumberLiteral,
    StringLiteral,
    TemplateLiteral,
    /// Anything else (`null`, `undefined`, objects, ...) that the rule ignores.
    Other,
}

/// One resolved type part: its flag and the rendered text used in messages.
#[derive(Clone, Debug)]
struct TypePart {
    flag: PartFlag,
    /// Rendered constituent text, retained for parity with the upstream
    /// diagnostic model (messages reference the redundant type name).
    #[allow(dead_code)]
    text: String,
}

/// A type node that resolved to two or more parts (definitely a union).
struct SeenUnion<'a> {
    parts: Vec<TypePart>,
    node: &'a LintNode,
}

/// Returns the rendered text for an AST keyword / literal part.
fn keyword_text(kind: &str) -> &'static str {
    match kind {
        "TSAnyKeyword" => "any",
        "TSBooleanKeyword" => "boolean",
        "TSNeverKeyword" => "never",
        "TSNumberKeyword" => "number",
        "TSStringKeyword" => "string",
        "TSUnknownKeyword" => "unknown",
        "TSBigIntKeyword" => "bigint",
        _ => "literal type",
    }
}

/// Maps a TS type-flag label (raw `__typePartFlags` fact) to a [`PartFlag`].
fn flag_from_label(label: &str) -> PartFlag {
    match label {
        "any" => PartFlag::Any,
        "error" => PartFlag::Error,
        "never" => PartFlag::Never,
        "unknown" => PartFlag::Unknown,
        "bigint" => PartFlag::BigInt,
        "boolean" => PartFlag::Boolean,
        "number" => PartFlag::Number,
        "string" => PartFlag::String,
        "bigintLiteral" => PartFlag::BigIntLiteral,
        "booleanLiteral" => PartFlag::BooleanLiteral,
        "numberLiteral" => PartFlag::NumberLiteral,
        "stringLiteral" => PartFlag::StringLiteral,
        "templateLiteral" => PartFlag::TemplateLiteral,
        _ => PartFlag::Other,
    }
}

/// Skips wrapping `TSParenthesizedType` nodes.
fn skip_parens(mut node: &LintNode) -> &LintNode {
    while node.kind == "TSParenthesizedType" {
        let Some(inner) = node.child("typeAnnotation") else {
            break;
        };
        node = inner;
    }
    node
}

/// Returns the rendered literal text for a `TSLiteralType` node, matching the
/// Go `ToString` behaviour for AST literal nodes.
fn literal_type_text(node: &LintNode) -> (PartFlag, String) {
    let Some(literal) = node.child("literal") else {
        return (PartFlag::Other, "literal type".to_owned());
    };
    // Template literal types: ESTree models them as TSTemplateLiteralType, or a
    // TSLiteralType whose literal is a TemplateLiteral.
    if literal.kind == "TemplateLiteral" || literal.kind == "TSTemplateLiteralType" {
        return (
            PartFlag::TemplateLiteral,
            "template literal type".to_owned(),
        );
    }
    match literal.kind.as_str() {
        "Literal" => match literal.fields.get("value") {
            Some(Value::Bool(value)) => (PartFlag::BooleanLiteral, value.to_string()),
            Some(Value::Number(value)) => (PartFlag::NumberLiteral, value.to_string()),
            Some(Value::String(value)) => (PartFlag::StringLiteral, value.clone()),
            _ => {
                // bigint literals carry a `bigint` field in ESTree.
                if let Some(bigint) = literal.field_str("bigint") {
                    (PartFlag::BigIntLiteral, format!("{bigint}n"))
                } else if literal.field_bool("value") == Some(true) {
                    (PartFlag::BooleanLiteral, "true".to_owned())
                } else {
                    (PartFlag::Other, "literal type".to_owned())
                }
            }
        },
        // `true` / `false` keywords occasionally surface directly.
        "TSTrueKeyword" => (PartFlag::BooleanLiteral, "true".to_owned()),
        "TSFalseKeyword" => (PartFlag::BooleanLiteral, "false".to_owned()),
        _ => (PartFlag::Other, "literal type".to_owned()),
    }
}

/// Reads the raw `__typePartFlags` checker fact for nodes the AST cannot resolve
/// locally (type references, mapped/indexed types, ...). Each entry is the flag
/// label and rendered text of one resolved union member.
fn fact_type_parts(node: &LintNode) -> Option<Vec<TypePart>> {
    let array = node.fields.get("__typePartFlags")?.as_array()?;
    let mut parts = Vec::with_capacity(array.len());
    for entry in array {
        let flag = entry
            .get("flag")
            .and_then(Value::as_str)
            .map(flag_from_label)
            .unwrap_or(PartFlag::Other);
        let text = entry
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        parts.push(TypePart { flag, text });
    }
    Some(parts)
}

/// Port of `getTypeNodeTypePartFlags`: returns the resolved type parts for a type
/// node, recursing into unions and consulting the checker fact when necessary.
fn type_node_part_flags(node: &LintNode) -> Vec<TypePart> {
    let node = skip_parens(node);

    if let Some(flag) = keyword_part_flag(node.kind.as_str()) {
        return vec![TypePart {
            flag,
            text: keyword_text(node.kind.as_str()).to_owned(),
        }];
    }

    if node.kind == "TSLiteralType" {
        let (flag, text) = literal_type_text(node);
        if flag != PartFlag::Other {
            return vec![TypePart { flag, text }];
        }
    }
    // Template literal types modeled as their own node kind.
    if node.kind == "TSTemplateLiteralType" {
        return vec![TypePart {
            flag: PartFlag::TemplateLiteral,
            text: "template literal type".to_owned(),
        }];
    }

    if node.kind == "TSUnionType" {
        let mut result = Vec::new();
        for sub in node.child_list("types").unwrap_or(&[]) {
            result.extend(type_node_part_flags(sub));
        }
        return result;
    }

    // Fall back to the checker-provided fact (type references and friends).
    if let Some(parts) = fact_type_parts(node) {
        return parts;
    }

    // No type info available: treat as an opaque non-redundant constituent.
    vec![TypePart {
        flag: PartFlag::Other,
        text: node
            .field_str("__typeText")
            .or_else(|| node.type_texts.first().map(String::as_str))
            .unwrap_or("")
            .to_owned(),
    }]
}

fn keyword_part_flag(kind: &str) -> Option<PartFlag> {
    match kind {
        "TSAnyKeyword" => Some(PartFlag::Any),
        "TSBigIntKeyword" => Some(PartFlag::BigInt),
        "TSBooleanKeyword" => Some(PartFlag::Boolean),
        "TSNeverKeyword" => Some(PartFlag::Never),
        "TSNumberKeyword" => Some(PartFlag::Number),
        "TSStringKeyword" => Some(PartFlag::String),
        "TSUnknownKeyword" => Some(PartFlag::Unknown),
        _ => None,
    }
}

/// Returns whether the union/intersection node sits in a function return type.
fn is_inside_return_type(node: &LintNode) -> bool {
    matches!(
        node.field_str("__parentKind"),
        Some(
            "FunctionDeclaration"
                | "FunctionExpression"
                | "ArrowFunctionExpression"
                | "TSDeclareFunction"
                | "TSEmptyBodyFunctionExpression"
                | "TSFunctionType"
                | "TSConstructorType"
                | "TSCallSignatureDeclaration"
                | "TSConstructSignatureDeclaration"
                | "TSMethodSignature"
                | "MethodDefinition"
        )
    )
}

/// Reports bottom/top types for an intersection constituent. Returns whether a
/// diagnostic was emitted (so the caller skips literal/primitive bookkeeping).
fn check_intersection_bottom_top(
    ctx: &mut RuleContext<'_>,
    part: &TypePart,
    type_node: &LintNode,
) -> bool {
    match part.flag {
        PartFlag::Any => {
            // "any" overrides; an error type (acts as any) gets its own message.
            ctx.report("overrides", type_node.range);
            true
        }
        PartFlag::Error => {
            ctx.report("errorTypeOverrides", type_node.range);
            true
        }
        PartFlag::Never => {
            ctx.report("overrides", type_node.range);
            true
        }
        PartFlag::Unknown => {
            ctx.report("overridden", type_node.range);
            true
        }
        _ => false,
    }
}

fn check_intersection(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let constituents = node.child_list("types").unwrap_or(&[]);

    let mut seen_bigint_literal: Vec<TypePart> = Vec::new();
    let mut seen_boolean_literal: Vec<TypePart> = Vec::new();
    let mut seen_number_literal: Vec<TypePart> = Vec::new();
    let mut seen_string_literal: Vec<TypePart> = Vec::new();

    let mut seen_bigint_primitive: Vec<&LintNode> = Vec::new();
    let mut seen_boolean_primitive: Vec<&LintNode> = Vec::new();
    let mut seen_number_primitive: Vec<&LintNode> = Vec::new();
    let mut seen_string_primitive: Vec<&LintNode> = Vec::new();

    let mut seen_unions: Vec<SeenUnion<'_>> = Vec::new();

    for type_node in constituents {
        let parts = type_node_part_flags(type_node);

        // 2+ parts on a single type node => it is definitely a union reference.
        if parts.len() >= 2 {
            seen_unions.push(SeenUnion {
                parts: parts.clone(),
                node: type_node,
            });
        }

        for part in &parts {
            if check_intersection_bottom_top(ctx, part, type_node) {
                continue;
            }

            if seen_unions.is_empty() {
                match part.flag {
                    PartFlag::BigIntLiteral => seen_bigint_literal.push(part.clone()),
                    PartFlag::BooleanLiteral => seen_boolean_literal.push(part.clone()),
                    PartFlag::NumberLiteral => seen_number_literal.push(part.clone()),
                    PartFlag::StringLiteral | PartFlag::TemplateLiteral => {
                        seen_string_literal.push(part.clone());
                    }
                    _ => {}
                }
            }

            match part.flag {
                PartFlag::BigInt => seen_bigint_primitive.push(type_node),
                PartFlag::Boolean => seen_boolean_primitive.push(type_node),
                PartFlag::Number => seen_number_primitive.push(type_node),
                PartFlag::String => seen_string_primitive.push(type_node),
                _ => {}
            }
        }
    }

    // Union member overridden by a primitive elsewhere in the intersection.
    if !seen_unions.is_empty()
        && (!seen_bigint_primitive.is_empty()
            || !seen_boolean_primitive.is_empty()
            || !seen_number_primitive.is_empty()
            || !seen_string_primitive.is_empty())
    {
        for union_type in &seen_unions {
            let mut primitive_matched = true;
            for value in &union_type.parts {
                let ok = match value.flag {
                    PartFlag::BigIntLiteral => !seen_bigint_primitive.is_empty(),
                    PartFlag::BooleanLiteral => !seen_boolean_primitive.is_empty(),
                    PartFlag::NumberLiteral => !seen_number_primitive.is_empty(),
                    PartFlag::StringLiteral | PartFlag::TemplateLiteral => {
                        !seen_string_primitive.is_empty()
                    }
                    _ => false,
                };
                if !ok {
                    primitive_matched = false;
                    break;
                }
            }
            if primitive_matched {
                ctx.report("primitiveOverridden", union_type.node.range);
            }
        }
    }

    if !seen_unions.is_empty() {
        return;
    }

    report_literals_override_primitive(ctx, &seen_bigint_literal, &seen_bigint_primitive);
    report_literals_override_primitive(ctx, &seen_boolean_literal, &seen_boolean_primitive);
    report_literals_override_primitive(ctx, &seen_number_literal, &seen_number_primitive);
    report_literals_override_primitive(ctx, &seen_string_literal, &seen_string_primitive);
}

fn report_literals_override_primitive(
    ctx: &mut RuleContext<'_>,
    literal_types: &[TypePart],
    primitive_nodes: &[&LintNode],
) {
    if literal_types.is_empty() {
        return;
    }
    for type_node in primitive_nodes {
        ctx.report("primitiveOverridden", type_node.range);
    }
}

/// Reports bottom/top types for a union constituent. Returns whether a
/// diagnostic was emitted.
fn check_union_bottom_top(
    ctx: &mut RuleContext<'_>,
    part: &TypePart,
    type_node: &LintNode,
    union_node: &LintNode,
) -> bool {
    match part.flag {
        PartFlag::Any => {
            ctx.report("overrides", type_node.range);
            true
        }
        PartFlag::Error => {
            ctx.report("errorTypeOverrides", type_node.range);
            true
        }
        PartFlag::Unknown => {
            ctx.report("overrides", type_node.range);
            true
        }
        PartFlag::Never => {
            // `never` in a function return type position is intentional.
            if is_inside_return_type(union_node) {
                return false;
            }
            ctx.report("overridden", type_node.range);
            true
        }
        _ => false,
    }
}

fn check_union(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let constituents = node.child_list("types").unwrap_or(&[]);

    // Maps from the constituent's start offset to (node, overridden literals).
    let mut overridden_bigint: BTreeMap<u32, (&LintNode, Vec<TypePart>)> = BTreeMap::new();
    let mut overridden_boolean: BTreeMap<u32, (&LintNode, Vec<TypePart>)> = BTreeMap::new();
    let mut overridden_number: BTreeMap<u32, (&LintNode, Vec<TypePart>)> = BTreeMap::new();
    let mut overridden_string: BTreeMap<u32, (&LintNode, Vec<TypePart>)> = BTreeMap::new();

    let mut seen_bigint_primitive = false;
    let mut seen_boolean_primitive = false;
    let mut seen_number_primitive = false;
    let mut seen_string_primitive = false;

    for type_node in constituents {
        let parts = type_node_part_flags(type_node);

        for part in &parts {
            if check_union_bottom_top(ctx, part, type_node, node) {
                continue;
            }

            match part.flag {
                PartFlag::BigIntLiteral => overridden_bigint
                    .entry(type_node.range.start)
                    .or_insert_with(|| (type_node, Vec::new()))
                    .1
                    .push(part.clone()),
                PartFlag::BooleanLiteral => overridden_boolean
                    .entry(type_node.range.start)
                    .or_insert_with(|| (type_node, Vec::new()))
                    .1
                    .push(part.clone()),
                PartFlag::NumberLiteral => overridden_number
                    .entry(type_node.range.start)
                    .or_insert_with(|| (type_node, Vec::new()))
                    .1
                    .push(part.clone()),
                PartFlag::StringLiteral | PartFlag::TemplateLiteral => overridden_string
                    .entry(type_node.range.start)
                    .or_insert_with(|| (type_node, Vec::new()))
                    .1
                    .push(part.clone()),
                _ => {}
            }

            match part.flag {
                PartFlag::BigInt => seen_bigint_primitive = true,
                PartFlag::Boolean => seen_boolean_primitive = true,
                PartFlag::Number => seen_number_primitive = true,
                PartFlag::String => seen_string_primitive = true,
                _ => {}
            }
        }
    }

    report_overridden_literals(ctx, seen_bigint_primitive, &overridden_bigint);
    report_overridden_literals(ctx, seen_boolean_primitive, &overridden_boolean);
    report_overridden_literals(ctx, seen_number_primitive, &overridden_number);
    report_overridden_literals(ctx, seen_string_primitive, &overridden_string);
}

fn report_overridden_literals(
    ctx: &mut RuleContext<'_>,
    seen_primitive: bool,
    overridden: &BTreeMap<u32, (&LintNode, Vec<TypePart>)>,
) {
    if !seen_primitive {
        return;
    }
    for (type_node, _literals) in overridden.values() {
        ctx.report("literalOverridden", type_node.range);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::NoRedundantTypeConstituentsRule;

    fn run(node: &LintNode) -> Vec<(String, (u32, u32))> {
        let rule = NoRedundantTypeConstituentsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|d| (d.message_id, (d.range.start, d.range.end)))
            .collect()
    }

    fn keyword(kind: &str, start: u32, end: u32) -> serde_json::Value {
        json!({ "kind": kind, "range": { "start": start, "end": end } })
    }

    fn number_literal(value: i64, start: u32, end: u32) -> serde_json::Value {
        json!({
            "kind": "TSLiteralType",
            "range": { "start": start, "end": end },
            "children": { "literal": { "kind": "Literal", "range": { "start": start, "end": end }, "fields": { "value": value } } }
        })
    }

    fn string_literal(value: &str, start: u32, end: u32) -> serde_json::Value {
        json!({
            "kind": "TSLiteralType",
            "range": { "start": start, "end": end },
            "children": { "literal": { "kind": "Literal", "range": { "start": start, "end": end }, "fields": { "value": value } } }
        })
    }

    // type T = number | any;  -> "overrides" on `any`.
    #[test]
    fn union_any_overrides() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 22 },
            "childLists": { "types": [
                keyword("TSNumberKeyword", 9, 15),
                keyword("TSAnyKeyword", 18, 21)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("overrides".to_owned(), (18, 21))]);
    }

    // type T = number | never; -> "overridden" on `never`.
    #[test]
    fn union_never_overridden() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 24 },
            "childLists": { "types": [
                keyword("TSNumberKeyword", 9, 15),
                keyword("TSNeverKeyword", 18, 23)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("overridden".to_owned(), (18, 23))]);
    }

    // type T = number | 0; -> "literalOverridden" on the literal `0`.
    #[test]
    fn union_literal_overridden_by_primitive() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 20 },
            "childLists": { "types": [
                keyword("TSNumberKeyword", 9, 15),
                number_literal(0, 18, 19)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("literalOverridden".to_owned(), (18, 19))]);
    }

    // type T = '' & string; -> "primitiveOverridden" on `string`.
    #[test]
    fn intersection_primitive_overridden_by_literal() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSIntersectionType",
            "range": { "start": 9, "end": 21 },
            "childLists": { "types": [
                string_literal("", 9, 11),
                keyword("TSStringKeyword", 14, 20)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("primitiveOverridden".to_owned(), (14, 20))]);
    }

    // type T = number & unknown; -> "overridden" on `unknown`.
    #[test]
    fn intersection_unknown_overridden() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSIntersectionType",
            "range": { "start": 9, "end": 26 },
            "childLists": { "types": [
                keyword("TSNumberKeyword", 9, 15),
                keyword("TSUnknownKeyword", 18, 25)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("overridden".to_owned(), (18, 25))]);
    }

    // Should NOT report: type T = number | null;
    #[test]
    fn union_number_null_ok() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 22 },
            "childLists": { "types": [
                keyword("TSNumberKeyword", 9, 15),
                keyword("TSNullKeyword", 18, 22)
            ] }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // Should NOT report: type T = 'a' | 'b';
    #[test]
    fn union_distinct_string_literals_ok() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 18 },
            "childLists": { "types": [
                string_literal("a", 9, 12),
                string_literal("b", 15, 18)
            ] }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // Should NOT report: type T = () => string | never;  (never in return type)
    #[test]
    fn union_never_in_return_type_ok() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 15, "end": 29 },
            "fields": { "__parentKind": "TSFunctionType" },
            "childLists": { "types": [
                keyword("TSStringKeyword", 15, 21),
                keyword("TSNeverKeyword", 24, 29)
            ] }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // type T = B | string; where B resolves (via checker fact) to "b" literal.
    #[test]
    fn union_reference_literal_overridden() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 20 },
            "childLists": { "types": [
                {
                    "kind": "TSTypeReference",
                    "range": { "start": 9, "end": 10 },
                    "fields": { "__typePartFlags": [ { "flag": "stringLiteral", "text": "\"b\"" } ] }
                },
                keyword("TSStringKeyword", 13, 19)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("literalOverridden".to_owned(), (9, 10))]);
    }

    // type U = T & string; where T resolves to a union 'a'|'b' (>=2 parts).
    #[test]
    fn intersection_union_reference_primitive_overridden() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSIntersectionType",
            "range": { "start": 9, "end": 20 },
            "childLists": { "types": [
                {
                    "kind": "TSTypeReference",
                    "range": { "start": 9, "end": 10 },
                    "fields": { "__typePartFlags": [
                        { "flag": "stringLiteral", "text": "\"a\"" },
                        { "flag": "stringLiteral", "text": "\"b\"" }
                    ] }
                },
                keyword("TSStringKeyword", 13, 19)
            ] }
        }))
        .unwrap();
        let out = run(&node);
        assert_eq!(out, vec![("primitiveOverridden".to_owned(), (9, 10))]);
    }
}

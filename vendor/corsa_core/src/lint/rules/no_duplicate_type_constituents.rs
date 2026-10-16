use std::collections::BTreeSet;

use serde_json::Value;

use super::super::{LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{child_list, rule_option_bool};

/// Type-aware rule that disallows duplicate union or intersection constituents.
///
/// Faithful port of tsgolint's `no-duplicate-type-constituents`. The rule walks
/// the constituents of a top-level union or intersection (recursing into
/// parenthesized constituents of the *same* kind, mirroring the upstream
/// `checkDuplicateRecursively` flattening) and reports the second and later
/// constituent that resolves to a type already seen. For unions whose parent is
/// an optional parameter, an explicit `undefined` constituent is additionally
/// reported as `unnecessary`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDuplicateTypeConstituentsRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "duplicate",
        description: "Union type constituent is duplicated with a previous constituent.",
    },
    RuleMessage {
        id: "unnecessary",
        description: "Explicit undefined is unnecessary on an optional parameter.",
    },
];

const LISTENERS: &[&str] = &["TSUnionType", "TSIntersectionType"];

impl RustLintRule for NoDuplicateTypeConstituentsRule {
    fn name(&self) -> &'static str {
        "no-duplicate-type-constituents"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow duplicate union or intersection constituents."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let options = Options::from_node(node);
        let kind = match node.kind.as_str() {
            "TSIntersectionType" => {
                if options.ignore_intersections {
                    return;
                }
                Kind::Intersection
            }
            "TSUnionType" => {
                if options.ignore_unions {
                    return;
                }
                Kind::Union
            }
            _ => return,
        };

        // Mirrors `unwindedParentType(node, kind) != nil`: skip a union or
        // intersection that is nested (through parenthesized types) inside another
        // of the SAME kind, because the outer listener already flattened it.
        if has_same_kind_ancestor(node) {
            return;
        }

        check_duplicate(ctx, node, kind);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Union,
    Intersection,
}

#[derive(Clone, Copy, Debug, Default)]
struct Options {
    ignore_intersections: bool,
    ignore_unions: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            ignore_intersections: rule_option_bool(node, "ignoreIntersections").unwrap_or(false),
            ignore_unions: rule_option_bool(node, "ignoreUnions").unwrap_or(false),
        }
    }
}

/// Returns whether an ancestor (looking only through parenthesized types) is a
/// union/intersection of the same kind as `node`.
///
/// Consumes the host fact `__sameKindAncestor` (a raw checker/AST observation
/// computed by walking `node`'s parents while they are `TSParenthesizedType`
/// and reporting whether the first non-parenthesized ancestor matches `node`'s
/// kind). The verdict (skip vs. check) stays here in Rust.
fn has_same_kind_ancestor(node: &LintNode) -> bool {
    node.field_bool("__sameKindAncestor").unwrap_or(false)
}

fn check_duplicate(ctx: &mut RuleContext<'_>, node: &LintNode, kind: Kind) {
    let parent_optional_param =
        kind == Kind::Union && node.field_bool("__parentOptionalParam").unwrap_or(false);

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for constituent in child_list(node, "types") {
        check_duplicate_recursively(ctx, kind, constituent, &mut seen, parent_optional_param);
    }
}

/// Recursively checks one constituent, mirroring upstream
/// `checkDuplicateRecursively`. Returns whether this constituent was reported as
/// a duplicate (used only for recursion symmetry; the fix selection here always
/// emits a removal suggestion, see parity notes).
fn check_duplicate_recursively(
    ctx: &mut RuleContext<'_>,
    kind: Kind,
    constituent: &LintNode,
    seen: &mut BTreeSet<String>,
    parent_optional_param: bool,
) -> bool {
    // `utils.IsIntrinsicErrorType(t)` short-circuit: when the host could not
    // resolve a non-error type id, the constituent never participates in
    // duplicate detection or caching. This is why `type T = A | A` (with `A`
    // unresolved) is NOT reported upstream.
    let type_id = resolved_type_id(constituent);

    if let Some(id) = type_id.as_deref() {
        if seen.contains(id) {
            report(ctx, constituent);
            return true;
        }
    }

    // `forEachNodeType`: for a union whose parent is an optional parameter, an
    // explicit `undefined` constituent is reported as `unnecessary`.
    if parent_optional_param && is_undefined_type(constituent) {
        report_unnecessary(ctx, constituent);
    }

    if let Some(id) = type_id {
        seen.insert(id);
    }

    // Flatten into same-kind constituents reached through parenthesized types.
    let Some(nested) = same_kind_constituents(constituent, kind) else {
        return false;
    };
    for child in nested {
        check_duplicate_recursively(ctx, kind, child, seen, parent_optional_param);
    }
    false
}

/// Returns the constituents of `node` when, after skipping parenthesized types,
/// it is a union/intersection of the same `kind`; otherwise `None`. Mirrors
/// `ast.SkipTypeParentheses` + the kind match in upstream.
fn same_kind_constituents(node: &LintNode, kind: Kind) -> Option<&[LintNode]> {
    let inner = skip_type_parentheses(node);
    match (kind, inner.kind.as_str()) {
        (Kind::Union, "TSUnionType") | (Kind::Intersection, "TSIntersectionType") => {
            inner.child_list("types")
        }
        _ => None,
    }
}

/// Mirrors `ast.SkipTypeParentheses`: unwraps nested `TSParenthesizedType`.
fn skip_type_parentheses(node: &LintNode) -> &LintNode {
    let mut current = node;
    while current.kind == "TSParenthesizedType" {
        match current.child("typeAnnotation") {
            Some(inner) => current = inner,
            None => break,
        }
    }
    current
}

/// Reads the host-provided stable resolved-type identity for a constituent.
///
/// Consumes the raw host fact `__resolvedTypeId` (a string). The host computes
/// it as: `const t = checker.getTypeAtLocation(constituentNode); if
/// (utils.isIntrinsicErrorType(t)) leave unset; else assign a per-source-file
/// stable id from a `Map<ts.Type, number>` (pointer identity, matching the
/// upstream `map[*checker.Type]*ast.Node`)`. The duplicate verdict stays in
/// Rust.
fn resolved_type_id(node: &LintNode) -> Option<String> {
    node.field_str("__resolvedTypeId")
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .or_else(|| syntax_type_id(node))
}

/// Reads whether the constituent's resolved type has the `undefined` type flag.
///
/// Consumes the raw host fact `__isUndefinedType` computed as
/// `utils.isTypeFlagSet(checker.getTypeAtLocation(constituentNode),
/// ts.TypeFlags.Undefined)`.
fn is_undefined_type(node: &LintNode) -> bool {
    node.field_bool("__isUndefinedType").unwrap_or(false) || node.kind == "TSUndefinedKeyword"
}

fn syntax_type_id(node: &LintNode) -> Option<String> {
    let node = skip_type_parentheses(node);
    if let Some(keyword) = keyword_type_id(node.kind.as_str()) {
        return Some(keyword.to_owned());
    }
    if node.kind == "TSLiteralType" {
        return literal_type_id(node);
    }
    if node.kind == "TSTemplateLiteralType" {
        return node
            .text
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|text| format!("template:{text}"));
    }
    None
}

fn keyword_type_id(kind: &str) -> Option<&'static str> {
    match kind {
        "TSAnyKeyword" => Some("keyword:any"),
        "TSBigIntKeyword" => Some("keyword:bigint"),
        "TSBooleanKeyword" => Some("keyword:boolean"),
        "TSNeverKeyword" => Some("keyword:never"),
        "TSNullKeyword" => Some("keyword:null"),
        "TSNumberKeyword" => Some("keyword:number"),
        "TSObjectKeyword" => Some("keyword:object"),
        "TSStringKeyword" => Some("keyword:string"),
        "TSSymbolKeyword" => Some("keyword:symbol"),
        "TSUndefinedKeyword" => Some("keyword:undefined"),
        "TSUnknownKeyword" => Some("keyword:unknown"),
        "TSVoidKeyword" => Some("keyword:void"),
        _ => None,
    }
}

fn literal_type_id(node: &LintNode) -> Option<String> {
    let Some(literal) = node.child("literal") else {
        return node
            .text
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|text| format!("literal:{text}"));
    };
    if literal.kind == "TemplateLiteral" || literal.kind == "TSTemplateLiteralType" {
        return literal
            .text
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|text| format!("template:{text}"));
    }
    match literal.kind.as_str() {
        "Literal" => match literal.fields.get("value") {
            Some(Value::Bool(value)) => Some(format!("literal:bool:{value}")),
            Some(Value::Number(value)) => Some(format!("literal:number:{value}")),
            Some(Value::String(value)) => Some(format!("literal:string:{value}")),
            _ => literal
                .field_str("bigint")
                .map(|bigint| format!("literal:bigint:{bigint}")),
        },
        "TSTrueKeyword" => Some("literal:bool:true".to_owned()),
        "TSFalseKeyword" => Some("literal:bool:false".to_owned()),
        _ => node
            .text
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|text| format!("literal:{text}")),
    }
}

fn report(ctx: &mut RuleContext<'_>, constituent: &LintNode) {
    let suggestion = LintSuggestion {
        message_id: "duplicate".to_owned(),
        message: "Remove the duplicated type constituent.".to_owned(),
        fixes: vec![LintFix::remove_range(constituent.range)],
    };
    ctx.report_with_suggestions("duplicate", constituent.range, vec![suggestion]);
}

fn report_unnecessary(ctx: &mut RuleContext<'_>, constituent: &LintNode) {
    let suggestion = LintSuggestion {
        message_id: "unnecessary".to_owned(),
        message: "Remove the unnecessary undefined constituent.".to_owned(),
        fixes: vec![LintFix::remove_range(constituent.range)],
    };
    ctx.report_with_suggestions("unnecessary", constituent.range, vec![suggestion]);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintDiagnostic, LintNode, RuleContext, RustLintRule};
    use super::NoDuplicateTypeConstituentsRule;

    fn run(node: &LintNode) -> Vec<LintDiagnostic> {
        let rule = NoDuplicateTypeConstituentsRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn constituent(start: u32, end: u32, type_id: &str) -> serde_json::Value {
        json!({
            "kind": "TSTypeReference",
            "range": { "start": start, "end": end },
            "fields": { "__resolvedTypeId": type_id }
        })
    }

    fn keyword(kind: &str, start: u32, end: u32, type_id: &str) -> serde_json::Value {
        json!({
            "kind": kind,
            "range": { "start": start, "end": end },
            "fields": { "__resolvedTypeId": type_id }
        })
    }

    // `type T = 1 | 1;` -> reports duplicate on the second literal.
    #[test]
    fn reports_duplicate_union_literal() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 14 },
            "childLists": {
                "types": [
                    keyword("TSLiteralType", 9, 10, "1"),
                    keyword("TSLiteralType", 13, 14, "1"),
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "duplicate");
        assert_eq!(diagnostics[0].range.start, 13);
        assert_eq!(diagnostics[0].range.end, 14);
    }

    // `type T = string | string;` -> reports duplicate even when the host does
    // not provide checker-backed type ids for syntax-only keyword nodes.
    #[test]
    fn reports_duplicate_primitive_keyword_without_checker_fact() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 24 },
            "childLists": {
                "types": [
                    { "kind": "TSStringKeyword", "range": { "start": 9, "end": 15 } },
                    { "kind": "TSStringKeyword", "range": { "start": 18, "end": 24 } }
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "duplicate");
        assert_eq!(diagnostics[0].range.start, 18);
        assert_eq!(diagnostics[0].range.end, 24);
    }

    // `type T = true & true;` -> reports duplicate on intersection.
    #[test]
    fn reports_duplicate_intersection() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSIntersectionType",
            "range": { "start": 9, "end": 20 },
            "childLists": {
                "types": [
                    keyword("TSLiteralType", 9, 13, "true"),
                    keyword("TSLiteralType", 16, 20, "true"),
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "duplicate");
    }

    // `(a?: string | undefined) => {};` -> explicit undefined is unnecessary.
    #[test]
    fn reports_unnecessary_undefined_on_optional_param() {
        let undefined_node = json!({
            "kind": "TSUndefinedKeyword",
            "range": { "start": 20, "end": 29 },
            "fields": { "__isUndefinedType": true, "__resolvedTypeId": "undefined" }
        });
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 11, "end": 29 },
            "fields": { "__parentOptionalParam": true },
            "childLists": {
                "types": [
                    keyword("TSStringKeyword", 11, 17, "string"),
                    undefined_node,
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unnecessary");
    }

    // `type T = A | A;` with A unresolved (error type) -> NOT reported.
    #[test]
    fn does_not_report_unresolved_alias_duplicates() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 14 },
            "childLists": {
                "types": [
                    // No __resolvedTypeId => error type => never duplicates.
                    json!({ "kind": "TSTypeReference", "range": { "start": 9, "end": 10 } }),
                    json!({ "kind": "TSTypeReference", "range": { "start": 13, "end": 14 } }),
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    // `type T = 1 | 2;` -> distinct types, NOT reported.
    #[test]
    fn does_not_report_distinct_union() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 14 },
            "childLists": {
                "types": [
                    keyword("TSLiteralType", 9, 10, "1"),
                    keyword("TSLiteralType", 13, 14, "2"),
                ]
            }
        }))
        .unwrap();

        assert!(run(&node).is_empty());
    }

    // `type T = A | A;` under `{ "ignoreUnions": true }` -> skipped.
    #[test]
    fn respects_ignore_unions_option() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 14 },
            "fields": { "__ruleOptions": [{ "ignoreUnions": true }] },
            "childLists": {
                "types": [
                    constituent(9, 10, "shared"),
                    constituent(13, 14, "shared"),
                ]
            }
        }))
        .unwrap();

        assert!(run(&node).is_empty());
    }

    // Nested same-kind union reached through parentheses is flattened:
    // `type A = (number | string) | number | string;` -> two duplicates.
    #[test]
    fn flattens_parenthesized_same_kind_constituents() {
        let paren = json!({
            "kind": "TSParenthesizedType",
            "range": { "start": 9, "end": 26 },
            "children": {
                "typeAnnotation": {
                    "kind": "TSUnionType",
                    "range": { "start": 10, "end": 25 },
                    "childLists": {
                        "types": [
                            keyword("TSNumberKeyword", 10, 16, "number"),
                            keyword("TSStringKeyword", 19, 25, "string"),
                        ]
                    }
                }
            }
        });
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 9, "end": 45 },
            "childLists": {
                "types": [
                    paren,
                    keyword("TSNumberKeyword", 29, 35, "number"),
                    keyword("TSStringKeyword", 38, 44, "string"),
                ]
            }
        }))
        .unwrap();

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|d| d.message_id == "duplicate"));
    }

    // A nested same-kind union should be skipped at its own listener invocation.
    #[test]
    fn skips_nested_same_kind_union() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "TSUnionType",
            "range": { "start": 10, "end": 25 },
            "fields": { "__sameKindAncestor": true },
            "childLists": {
                "types": [
                    constituent(10, 11, "shared"),
                    constituent(14, 15, "shared"),
                ]
            }
        }))
        .unwrap();

        assert!(run(&node).is_empty());
    }
}

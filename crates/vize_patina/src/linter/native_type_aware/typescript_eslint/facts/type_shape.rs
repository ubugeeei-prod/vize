//! Which expressions get the constrained `__type` and `__typeContext` facts.
use super::queries::Planner;
use crate::linter::corsa_session::{ExpressionQuery, PredicateRequest, ShapeRequest};
use corsa::lint::LintNode;
use vize_l0::FxHashMap as HashMap;

/// Attach shape requests to the queries of the operands the shape rules read
/// (conditions, comparison and logical operands, member objects, callees,
/// first arguments, unary operands and template interpolations), merged per
/// range so duplicate queries agree.
///
/// Type predicates cost a signature lookup per callee type, so `predicates`
/// (no-unnecessary-condition's configured `checkTypePredicates`) gates them.
pub(super) fn request(nodes: &[LintNode], predicates: bool, plan: &mut Planner) {
    let mut wanted = HashMap::<(u32, u32), ShapeRequest>::default();
    for node in nodes {
        walk(
            node,
            &mut wanted,
            ShapeRequest::default(),
            false,
            predicates,
        );
    }
    for ((start, end), request) in wanted {
        let mut query = ExpressionQuery::at(start, end);
        query.shape = Some(request);
        plan.push(query);
    }
}

fn walk(
    node: &LintNode,
    wanted: &mut HashMap<(u32, u32), ShapeRequest>,
    request: ShapeRequest,
    operand: bool,
    predicates: bool,
) {
    let predicate = if predicates {
        call_predicate_request(node)
    } else {
        None
    };
    if operand || predicate.is_some() {
        let entry = wanted
            .entry((node.range.start, node.range.end))
            .or_default();
        if entry.member.is_none() {
            entry.member = request.member;
        }
        entry.call |= request.call;
        entry.array |= request.array;
        entry.returns |= request.returns;
        if entry.keys_from.is_none() {
            entry.keys_from = request.keys_from;
        }
        if entry.predicate.is_none() {
            entry.predicate = predicate;
        }
    }
    for (key, child) in &node.children {
        if is_name_position(node, key) {
            continue;
        }
        let operand = is_value_expression(&child.kind) && is_operand(node, key);
        walk(child, wanted, child_request(node, key), operand, predicates);
    }
    for (key, children) in &node.child_lists {
        for (index, child) in children.iter().enumerate() {
            // `${T}` in a template literal type is the only type node rules read.
            let operand = match (node.kind.as_str(), key.as_str()) {
                ("TSTemplateLiteralType", "types") => true,
                ("TemplateLiteral", "expressions") => true,
                ("CallExpression", "arguments") => index == 0,
                _ => false,
            } && (is_value_expression(&child.kind)
                || node.kind == "TSTemplateLiteralType");
            let request = ShapeRequest {
                returns: index == 0 && key == "arguments" && is_predicate_call(node),
                ..ShapeRequest::default()
            };
            walk(child, wanted, request, operand, predicates);
        }
    }
}

/// The callee and the arguments before the first spread of a call, whose
/// type predicate `checkTypePredicates` reads.
fn call_predicate_request(node: &LintNode) -> Option<PredicateRequest> {
    if node.kind != "CallExpression" {
        return None;
    }
    let callee = node.child("callee")?;
    let arguments = node
        .child_list("arguments")
        .unwrap_or(&[])
        .iter()
        .take_while(|argument| argument.kind != "SpreadElement")
        .map(|argument| (argument.range.start, argument.range.end))
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        return None;
    }
    Some(PredicateRequest {
        callee: (callee.range.start, callee.range.end),
        arguments,
    })
}

/// Positions whose type the shape rules read.
fn is_operand(parent: &LintNode, key: &str) -> bool {
    match (parent.kind.as_str(), key) {
        (
            "IfStatement"
            | "WhileStatement"
            | "DoWhileStatement"
            | "ForStatement"
            | "ConditionalExpression"
            | "SwitchCase",
            "test",
        ) => true,
        ("SwitchStatement", "discriminant") => true,
        ("LogicalExpression" | "BinaryExpression" | "AssignmentExpression", "left" | "right") => {
            true
        }
        ("UnaryExpression" | "ChainExpression", "argument" | "expression") => true,
        ("MemberExpression", "object") => true,
        ("MemberExpression", "property") => parent.field_bool("computed") == Some(true),
        ("CallExpression", "callee") => true,
        // Array-predicate callbacks: `x => cond` and `x => { return cond; }`.
        ("ArrowFunctionExpression", "body") | ("ReturnStatement", "argument") => true,
        _ => false,
    }
}

/// Static property names are not expressions with a type of their own.
fn is_name_position(parent: &LintNode, key: &str) -> bool {
    let computed = parent.field_bool("computed") == Some(true);
    match key {
        "property" => parent.kind == "MemberExpression" && !computed,
        "key" => !computed,
        _ => false,
    }
}

fn child_request(parent: &LintNode, key: &str) -> ShapeRequest {
    match (parent.kind.as_str(), key) {
        ("MemberExpression", "object") => {
            let computed = parent.field_bool("computed") == Some(true);
            let member = if computed {
                None
            } else {
                parent
                    .child("property")
                    .and_then(|property| property.text.as_deref())
                    .map(vize_l0::String::from)
            };
            let predicate = member
                .as_deref()
                .is_some_and(|name| ARRAY_PREDICATES.contains(&name));
            let keys_from = computed
                .then(|| parent.child("property"))
                .flatten()
                .map(|property| (property.range.start, property.range.end));
            ShapeRequest {
                member,
                array: computed || predicate,
                keys_from,
                ..ShapeRequest::default()
            }
        }
        ("CallExpression", "callee") => ShapeRequest {
            call: true,
            ..ShapeRequest::default()
        },
        _ => ShapeRequest::default(),
    }
}

/// `arr.filter(cb)` and friends, whose callback return types are checked.
const ARRAY_PREDICATES: &[&str] = &[
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "some",
];

fn is_predicate_call(node: &LintNode) -> bool {
    node.kind == "CallExpression"
        && node
            .child("callee")
            .filter(|callee| callee.kind == "MemberExpression")
            .and_then(|callee| callee.child("property"))
            .and_then(|property| property.field_str("name"))
            .is_some_and(|name| ARRAY_PREDICATES.contains(&name))
}

fn is_value_expression(kind: &str) -> bool {
    if kind.ends_with("Expression") || kind.ends_with("Literal") {
        return !matches!(kind, "TSTemplateLiteralType");
    }
    matches!(
        kind,
        "Identifier" | "ThisExpression" | "Super" | "PrivateIdentifier"
    )
}

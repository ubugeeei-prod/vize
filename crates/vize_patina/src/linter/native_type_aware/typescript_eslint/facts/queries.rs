//! Which checker facts to request for which nodes.
//!
//! Every visited node gets its type text and flags. Structural shapes and
//! member-method facts cost extra checker requests, so they are requested only
//! for the positions an enabled rule inspects.
use crate::linter::corsa_session::{ExpressionQuery, detail};
use corsa::lint::LintNode;
use serde_json::Value;
use std::collections::HashMap;

/// Shape details (`detail::*`, `0` for none) the enabled rules need per
/// position, and whether unbound-method needs member facts.
#[derive(Clone, Copy, Default)]
struct Needs {
    /// `+` / `+=` operands (restrict-plus-operands, no-base-to-string).
    plus_operands: u8,
    /// Template literal expressions (restrict-template-expressions,
    /// no-base-to-string).
    template_expressions: u8,
    /// First call/`new` arguments: the `String()` argument (no-base-to-string)
    /// and eval-like handlers (no-implied-eval).
    first_argument: u8,
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
    members: bool,
}

impl Needs {
    fn from_rules(rules: &[&str]) -> Self {
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

/// Builds one query per distinct range for `nodes`, for the enabled corsa
/// rules.
pub(in super::super) fn queries(nodes: &[LintNode], rules: &[&str]) -> Vec<ExpressionQuery> {
    let needs = Needs::from_rules(rules);
    let mut queries = Vec::new();
    let mut seen = HashMap::new();
    for node in nodes {
        collect(node, needs, 0, &mut queries, &mut seen);
    }
    queries
}

fn query(node: &LintNode) -> ExpressionQuery {
    ExpressionQuery {
        start: node.range.start,
        end: node.range.end,
        symbols: node.kind == "Identifier",
        signatures: false,
        shape: 0,
        members: Vec::new(),
    }
}

fn collect(
    node: &LintNode,
    needs: Needs,
    shape: u8,
    queries: &mut Vec<ExpressionQuery>,
    seen: &mut HashMap<(u32, u32), usize>,
) {
    let mut own = query(node);
    own.shape = shape;
    if needs.members && node.kind == "ObjectPattern" {
        own.members = pattern_keys(node);
    }
    push(queries, seen, own);
    if node.kind == "CallExpression" {
        if let Some(callee) = node.child("callee") {
            let mut callee_query = query(callee);
            callee_query.symbols = false;
            callee_query.signatures = true;
            push(queries, seen, callee_query);
        }
    }
    if needs.members {
        collect_member_queries(node, queries, seen);
    }
    for (key, child) in &node.children {
        collect(
            child,
            needs,
            wants_shape(needs, node, key, 0),
            queries,
            seen,
        );
    }
    for (key, children) in &node.child_lists {
        for (index, child) in children.iter().enumerate() {
            collect(
                child,
                needs,
                wants_shape(needs, node, key, index),
                queries,
                seen,
            );
        }
    }
}

/// unbound-method reads the accessed property on the receiver's type, and
/// the destructured properties on a pattern's initializer.
fn collect_member_queries(
    node: &LintNode,
    queries: &mut Vec<ExpressionQuery>,
    seen: &mut HashMap<(u32, u32), usize>,
) {
    if node.kind == "MemberExpression" && !is_receiver_position(node) {
        if let (Some(object), Some(name)) = (node.child("object"), accessed_name(node)) {
            let mut object_query = query(object);
            object_query.members = vec![name];
            push(queries, seen, object_query);
        }
    }
    if node.kind == "ObjectPattern" {
        if let Some(range) = node.fields.get("__initRange") {
            let start = range.get("start").and_then(Value::as_u64);
            let end = range.get("end").and_then(Value::as_u64);
            if let (Some(start), Some(end)) = (start, end) {
                push(
                    queries,
                    seen,
                    ExpressionQuery {
                        start: start as u32,
                        end: end as u32,
                        symbols: node.field_str("__initName").is_some(),
                        signatures: false,
                        shape: 0,
                        members: pattern_keys(node),
                    },
                );
            }
        }
    }
}

/// `obj.method()` and `a.b.c` never unbind `this`; skip their checker work.
fn is_receiver_position(node: &LintNode) -> bool {
    matches!(
        node.field_str("__parentKind"),
        Some("MemberExpression") | Some("CallExpression")
    ) && node
        .fields
        .get("__ancestorChain")
        .and_then(Value::as_array)
        .and_then(|chain| chain.first())
        .and_then(|parent| parent.get("childKey"))
        .and_then(Value::as_str)
        .is_some_and(|key| matches!(key, "callee" | "object"))
}

/// The statically known property name of a member access.
fn accessed_name(node: &LintNode) -> Option<String> {
    let property = node.child("property")?;
    if node.field_bool("computed") == Some(true) {
        return match property.fields.get("value")? {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        };
    }
    (property.kind == "Identifier")
        .then(|| property.field_str("name").map(str::to_owned))
        .flatten()
}

fn pattern_keys(node: &LintNode) -> Vec<String> {
    node.child_list("properties")
        .unwrap_or(&[])
        .iter()
        .filter(|property| {
            property.kind == "Property" && property.field_bool("computed") != Some(true)
        })
        .filter_map(|property| property.child("key"))
        .filter(|key| key.kind == "Identifier")
        .filter_map(|key| key.field_str("name").map(str::to_owned))
        .collect()
}

/// One query per range; repeated ranges merge what they ask for.
fn push(
    queries: &mut Vec<ExpressionQuery>,
    seen: &mut HashMap<(u32, u32), usize>,
    query: ExpressionQuery,
) {
    if let Some(existing) = seen
        .get(&(query.start, query.end))
        .and_then(|&index| queries.get_mut(index))
    {
        existing.symbols |= query.symbols;
        existing.signatures |= query.signatures;
        existing.shape |= query.shape;
        for member in query.members {
            if !existing.members.contains(&member) {
                existing.members.push(member);
            }
        }
        return;
    }
    seen.insert((query.start, query.end), queries.len());
    queries.push(query);
}

/// Shape details an enabled rule needs for a child position.
fn wants_shape(needs: Needs, parent: &LintNode, key: &str, index: usize) -> u8 {
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

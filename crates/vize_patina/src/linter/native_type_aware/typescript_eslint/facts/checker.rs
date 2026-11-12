//! Checker facts beyond rendered type text: type details on every node,
//! resolved parameter types at calls, and the return types a function's
//! signatures declare (for `return` statements and arrow expression bodies).
use super::destructuring::{destructuring, pattern_paths, stamp_destructuring};
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Argument positions loaded beyond the argument count, so a spread tuple can
/// still be matched against the parameters after it.
const SPREAD_SLACK: u32 = 8;

/// Which checker facts the enabled rules read, so nothing else is asked for.
#[derive(Clone, Copy, Debug, Default)]
pub(in super::super) struct Needs {
    /// `__typeDetail` on every queried node.
    pub details: bool,
    /// Resolved parameter types at calls.
    pub calls: bool,
    /// Signature return types for `return` and arrow expression bodies.
    pub returns: bool,
    /// Contextual types of object-literal property keys.
    pub property_keys: bool,
    /// Member types reached by destructuring patterns.
    pub destructuring: bool,
    /// Contextual types of array elements and object property values.
    pub literal_members: bool,
    /// The `switch` discriminant of a `case`.
    pub switch_cases: bool,
    /// The extended class of a `super(...)` call.
    pub super_calls: bool,
}

impl Needs {
    /// The facts the given corsa rules read.
    pub(in super::super) fn for_rules(rules: &[&str]) -> Self {
        let any = |names: &[&str]| rules.iter().any(|rule| names.contains(rule));
        Self {
            details: any(&[
                "no-mixed-enums",
                "no-unsafe-argument",
                "no-unsafe-assignment",
                "no-unsafe-call",
                "no-unsafe-enum-assignment",
                "no-unsafe-enum-comparison",
                "no-unsafe-member-access",
                "no-unsafe-return",
                "no-unsafe-unary-minus",
            ]),
            calls: any(&["no-unsafe-argument", "no-unsafe-enum-assignment"]),
            returns: any(&["no-unsafe-return", "no-unsafe-enum-assignment"]),
            property_keys: any(&["no-unsafe-assignment"]),
            destructuring: any(&["no-unsafe-assignment"]),
            literal_members: any(&["no-unsafe-enum-assignment"]),
            switch_cases: any(&["no-unsafe-enum-comparison"]),
            super_calls: any(&["no-unsafe-call"]),
        }
    }
}

/// Queries for the listener node itself (call signature, own function returns)
/// and, for a `return`, its enclosing function.
pub(super) fn listener_queries(node: &LintNode, queries: &mut Vec<ExpressionQuery>, needs: Needs) {
    if needs.destructuring {
        if let Some((sender, pattern)) = destructuring(node) {
            let mut targets = Vec::new();
            pattern_paths(pattern, &mut Vec::new(), &mut targets);
            if !targets.is_empty() {
                let mut query = query(sender.range.start, sender.range.end, 0, None);
                query.detail = needs.details;
                query.paths = targets.into_iter().map(|(_, path)| path).collect();
                queries.push(query);
            }
        }
    }
    let mut push =
        |start: u32, end: u32, call_positions: u32, returns: Option<bool>, contextual| {
            let mut query = query(start, end, call_positions, returns);
            query.contextual = contextual;
            query.detail = needs.details;
            queries.push(query);
        };
    match node.kind.as_str() {
        "SwitchCase" if needs.switch_cases => {
            if let Some((start, end)) = switch_discriminant(node) {
                push(start, end, 0, None, false);
            }
        }
        "Property" if needs.property_keys => {
            if let Some(key) = contextual_key(node) {
                push(key.range.start, key.range.end, 0, None, true);
            }
        }
        "ArrayExpression" if needs.literal_members => {
            for element in node.child_list("elements").unwrap_or(&[]) {
                push(element.range.start, element.range.end, 0, None, true);
            }
        }
        "ObjectExpression" if needs.literal_members => {
            push(node.range.start, node.range.end, 0, None, true);
            for value in object_values(node) {
                push(value.range.start, value.range.end, 0, None, true);
            }
        }
        "CallExpression" | "NewExpression" | "TaggedTemplateExpression" => {
            if needs.calls {
                let arguments = call_arguments(node);
                push(
                    node.range.start,
                    node.range.end,
                    arguments + SPREAD_SLACK,
                    None,
                    false,
                );
            }
            if needs.super_calls {
                if let Some((start, end)) = super_call_base(node) {
                    push(start, end, 0, None, false);
                }
            }
        }
        "ArrowFunctionExpression" if needs.returns && has_expression_body(node) => {
            let contextual = contextual_parent(
                node.field_str("__parentKind"),
                node.field_str("__keyInParent"),
            );
            push(node.range.start, node.range.end, 0, Some(contextual), false);
        }
        "ReturnStatement" if needs.returns => {
            if let Some(function) = enclosing_function(node) {
                push(
                    function.start,
                    function.end,
                    0,
                    Some(function.contextual),
                    false,
                );
            }
        }
        _ => {}
    }
}

fn object_values(node: &LintNode) -> impl Iterator<Item = &LintNode> {
    node.child_list("properties")
        .unwrap_or(&[])
        .iter()
        .filter(|property| property.kind == "Property")
        .filter_map(|property| property.child("value"))
}

/// Facts answered by [`listener_queries`], keyed by query range.
#[derive(Default)]
pub(super) struct ListenerFacts<'a> {
    pub plain: HashMap<(u32, u32), &'a ExpressionFact>,
    pub calls: HashMap<(u32, u32), &'a ExpressionFact>,
    pub returns: HashMap<(u32, u32), &'a ExpressionFact>,
    pub contextuals: HashMap<(u32, u32), &'a ExpressionFact>,
    pub destructures: HashMap<(u32, u32), &'a ExpressionFact>,
}

impl<'a> ListenerFacts<'a> {
    pub(super) fn record(&mut self, query: &ExpressionQuery, fact: &'a ExpressionFact) {
        let key = (query.start, query.end);
        self.plain.entry(key).or_insert(fact);
        if query.call_positions > 0 {
            self.calls.insert(key, fact);
        }
        if query.function_returns.is_some() {
            self.returns.insert(key, fact);
        }
        if query.contextual {
            self.contextuals.insert(key, fact);
        }
        if !query.paths.is_empty() {
            self.destructures.insert(key, fact);
        }
    }
}

/// Stamps the checker facts gathered by [`listener_queries`].
pub(super) fn stamp_listener(node: &mut LintNode, facts: &ListenerFacts<'_>) {
    stamp_destructuring(node, facts);
    let ListenerFacts {
        calls,
        returns,
        contextuals,
        ..
    } = facts;
    match node.kind.as_str() {
        "SwitchCase" => {
            let detail = switch_discriminant(node)
                .and_then(|range| facts.plain.get(&range))
                .and_then(|fact| fact.detail.clone());
            if let Some(detail) = detail {
                node.fields
                    .insert("__switchDiscriminantDetail".into(), detail);
            }
        }
        "ArrayExpression" => {
            if let Some(elements) = node.child_lists.get_mut("elements") {
                for element in elements {
                    stamp_contextual(element, contextuals);
                }
            }
        }
        "ObjectExpression" => {
            stamp_contextual(node, contextuals);
            if let Some(properties) = node.child_lists.get_mut("properties") {
                for property in properties {
                    if property.kind == "Property" {
                        if let Some(value) = property.children.get_mut("value") {
                            stamp_contextual(value, contextuals);
                        }
                    }
                }
            }
        }
        "Property" => {
            let contextual = contextual_key(node)
                .and_then(|key| contextuals.get(&(key.range.start, key.range.end)))
                .and_then(|fact| fact.contextual.clone());
            if let Some(contextual) = contextual {
                node.fields.insert("__keyContextualType".into(), contextual);
            }
        }
        "CallExpression" | "NewExpression" | "TaggedTemplateExpression" => {
            if let Some(base) = super_call_base(node).and_then(|range| facts.plain.get(&range)) {
                type_super_callee(node, base);
            }
            if let Some(call) = calls
                .get(&(node.range.start, node.range.end))
                .and_then(|fact| fact.call.as_ref())
            {
                node.fields.insert("__callParameters".into(), call.clone());
            }
        }
        "ArrowFunctionExpression" => {
            let facts = returns
                .get(&(node.range.start, node.range.end))
                .and_then(|fact| fact.returns.as_ref());
            if let Some(facts) = facts {
                let has_return_type = node
                    .fields
                    .get("returnType")
                    .is_some_and(|annotation| !annotation.is_null())
                    || node.child("returnType").is_some();
                let is_async = node.field_bool("async").unwrap_or(false);
                insert_returns(node, facts.clone(), has_return_type, is_async);
            }
        }
        "ReturnStatement" => {
            let Some(function) = enclosing_function(node) else {
                return;
            };
            if let Some(facts) = returns
                .get(&(function.start, function.end))
                .and_then(|fact| fact.returns.as_ref())
            {
                insert_returns(
                    node,
                    facts.clone(),
                    function.has_return_type,
                    function.is_async,
                );
            }
        }
        _ => {}
    }
}

fn stamp_contextual(node: &mut LintNode, contextuals: &HashMap<(u32, u32), &ExpressionFact>) {
    if let Some(contextual) = contextuals
        .get(&(node.range.start, node.range.end))
        .and_then(|fact| fact.contextual.clone())
    {
        node.fields.insert("__contextualType".into(), contextual);
    }
}

/// Copies a node's checker type detail into `__typeDetail`.
pub(super) fn stamp_detail(node: &mut LintNode, fact: &ExpressionFact) {
    if let Some(detail) = &fact.detail {
        node.fields.insert("__typeDetail".into(), detail.clone());
    }
}

fn insert_returns(node: &mut LintNode, facts: Value, has_return_type: bool, is_async: bool) {
    node.fields.insert("__functionReturns".into(), facts);
    node.fields
        .insert("__functionHasReturnType".into(), json!(has_return_type));
    node.fields
        .insert("__functionIsAsync".into(), json!(is_async));
}

fn query(
    start: u32,
    end: u32,
    call_positions: u32,
    function_returns: Option<bool>,
) -> ExpressionQuery {
    ExpressionQuery {
        start,
        end,
        symbols: false,
        signatures: false,
        call_positions,
        function_returns,
        contextual: false,
        detail: false,
        paths: Vec::new(),
    }
}

/// typescript-eslint asks the checker for a property's contextual type only
/// through an identifier key of an object literal (not a pattern).
fn contextual_key(node: &LintNode) -> Option<&LintNode> {
    if node.field_str("__parentKind") == Some("ObjectPattern")
        || node.field_bool("computed") == Some(true)
    {
        return None;
    }
    node.child("key").filter(|key| key.kind == "Identifier")
}

/// The `switch (discriminant)` a case compares against.
fn switch_discriminant(node: &LintNode) -> Option<(u32, u32)> {
    let ancestors = node.fields.get("__ancestorFacts")?.as_array()?;
    let switch = ancestors
        .iter()
        .rev()
        .find(|ancestor| ancestor.get("kind").and_then(Value::as_str) == Some("SwitchStatement"))?;
    range_of(switch.get("discriminant")?)
}

fn range_of(value: &Value) -> Option<(u32, u32)> {
    let start = u32::try_from(value.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(value.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

/// The extended class expression of the class whose constructor makes the
/// `super(...)` call. The checker answers `any` for the `super` keyword itself,
/// while typescript-eslint sees the base constructor type.
fn super_call_base(node: &LintNode) -> Option<(u32, u32)> {
    if node.kind != "CallExpression" || node.child("callee")?.kind != "Super" {
        return None;
    }
    let ancestors = node.fields.get("__ancestorFacts")?.as_array()?;
    let class = ancestors.iter().rev().find(|ancestor| {
        matches!(
            ancestor.get("kind").and_then(Value::as_str),
            Some("ClassDeclaration" | "ClassExpression")
        )
    })?;
    range_of(class.get("superClass")?)
}

fn type_super_callee(node: &mut LintNode, base: &ExpressionFact) {
    let Some(callee) = node.children.get_mut("callee") else {
        return;
    };
    callee
        .fields
        .insert("__typedFromSuperClass".into(), json!(true));
    callee.type_texts.clone_from(&base.type_texts);
    callee.property_names.clone_from(&base.property_names);
    match &base.detail {
        Some(detail) => {
            callee.fields.insert("__typeDetail".into(), detail.clone());
        }
        None => {
            callee.fields.remove("__typeDetail");
        }
    }
}

fn call_arguments(node: &LintNode) -> u32 {
    let count = if node.kind == "TaggedTemplateExpression" {
        node.child("quasi")
            .and_then(|quasi| quasi.child_list("expressions"))
            .map_or(0, |expressions| expressions.len() + 1)
    } else {
        node.child_list("arguments").map_or(0, <[LintNode]>::len)
    };
    u32::try_from(count).unwrap_or(u32::MAX)
}

fn has_expression_body(node: &LintNode) -> bool {
    node.child("body")
        .is_some_and(|body| body.kind != "BlockStatement")
}

/// typescript-eslint's `getContextualType` only consults the checker for these
/// parents; elsewhere a function is judged by its own type.
fn contextual_parent(parent: Option<&str>, key: Option<&str>) -> bool {
    matches!(
        (parent, key),
        (Some("CallExpression" | "NewExpression"), Some("arguments"))
            | (Some("VariableDeclarator"), Some("init"))
            | (Some("PropertyDefinition"), Some("value"))
            | (
                Some("AssignmentExpression" | "AssignmentPattern"),
                Some("right")
            )
            | (Some("JSXExpressionContainer"), Some("expression"))
            | (Some("TemplateLiteral"), Some("expressions"))
    )
}

struct EnclosingFunction {
    start: u32,
    end: u32,
    contextual: bool,
    has_return_type: bool,
    is_async: bool,
}

fn enclosing_function(node: &LintNode) -> Option<EnclosingFunction> {
    let ancestors = node.fields.get("__ancestorFacts")?.as_array()?;
    let index = ancestors.iter().rposition(|ancestor| {
        matches!(
            ancestor.get("kind").and_then(Value::as_str),
            Some("FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression")
        )
    })?;
    let function = ancestors.get(index)?;
    let parent = index
        .checked_sub(1)
        .and_then(|parent| ancestors.get(parent))
        .and_then(|parent| parent.get("kind"))
        .and_then(Value::as_str);
    let key = function.get("keyInParent").and_then(Value::as_str);
    Some(EnclosingFunction {
        start: u32::try_from(function.get("start")?.as_u64()?).ok()?,
        end: u32::try_from(function.get("end")?.as_u64()?).ok()?,
        contextual: function.get("kind").and_then(Value::as_str) != Some("FunctionDeclaration")
            && contextual_parent(parent, key),
        has_return_type: function
            .get("hasReturnType")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_async: function
            .get("async")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

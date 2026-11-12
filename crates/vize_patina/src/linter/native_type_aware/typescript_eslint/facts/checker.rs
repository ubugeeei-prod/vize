//! Checker facts beyond rendered type text: type details at the positions
//! the no-unsafe and enum rules read, resolved parameter types at calls, and
//! the return types a function's signatures declare (for `return` statements
//! and arrow expression bodies).
mod syntax;
use super::destructuring::{destructuring, pattern_paths, stamp_destructuring};
use super::queries::Planner;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use syntax::{call_arguments, has_expression_body, range_of, super_call_base, type_super_callee};
use vize_l0::FxHashMap;

/// Argument positions loaded beyond the argument count, so a spread tuple can
/// still be matched against the parameters after it.
const SPREAD_SLACK: u32 = 8;

/// Which checker facts the enabled rules read, so nothing else is asked for.
#[derive(Clone, Copy, Debug, Default)]
pub(in super::super) struct Needs {
    /// `__typeDetail`, per rule that reads it.
    pub detail: DetailRules,
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

/// The enabled rules that read `__typeDetail`.
#[derive(Clone, Copy, Debug, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one switch per rule that reads type details"
)]
pub(in super::super) struct DetailRules {
    mixed_enums: bool,
    argument: bool,
    assignment: bool,
    call: bool,
    enum_assignment: bool,
    enum_comparison: bool,
    member_access: bool,
    returns: bool,
    unary_minus: bool,
}

impl DetailRules {
    fn any(self) -> bool {
        self.mixed_enums
            || self.argument
            || self.assignment
            || self.call
            || self.enum_assignment
            || self.enum_comparison
            || self.member_access
            || self.returns
            || self.unary_minus
    }
}

impl Needs {
    /// The facts the given corsa rules read.
    pub(in super::super) fn for_rules(rules: &[&str]) -> Self {
        let has = |name: &str| rules.contains(&name);
        let any = |names: &[&str]| names.iter().any(|name| has(name));
        Self {
            detail: DetailRules {
                mixed_enums: has("no-mixed-enums"),
                argument: has("no-unsafe-argument"),
                assignment: has("no-unsafe-assignment"),
                call: has("no-unsafe-call"),
                enum_assignment: has("no-unsafe-enum-assignment"),
                enum_comparison: has("no-unsafe-enum-comparison"),
                member_access: has("no-unsafe-member-access"),
                returns: has("no-unsafe-return"),
                unary_minus: has("no-unsafe-unary-minus"),
            },
            calls: any(&["no-unsafe-argument", "no-unsafe-enum-assignment"]),
            returns: any(&["no-unsafe-return", "no-unsafe-enum-assignment"]),
            property_keys: has("no-unsafe-assignment"),
            destructuring: has("no-unsafe-assignment"),
            literal_members: has("no-unsafe-enum-assignment"),
            switch_cases: has("no-unsafe-enum-comparison"),
            super_calls: has("no-unsafe-call"),
        }
    }
}

/// Whether an enabled rule reads `__typeDetail` off the `key` child of
/// `parent`. Each rule reads its listener's direct operands (see the rules'
/// `TypeDetail::of` calls); nothing deeper is read except through these
/// same positions on nested listeners.
pub(super) fn detail_at(needs: Needs, parent: &LintNode, key: &str) -> bool {
    let rules = needs.detail;
    if !rules.any() {
        return false;
    }
    let operator = parent.field_str("operator");
    match (parent.kind.as_str(), key) {
        ("TSEnumMember", "initializer") => rules.mixed_enums,
        ("CallExpression" | "NewExpression", "callee") | ("TaggedTemplateExpression", "tag") => {
            rules.argument || rules.call
        }
        ("CallExpression" | "NewExpression", "arguments") | ("TemplateLiteral", "expressions") => {
            rules.argument || rules.enum_assignment
        }
        ("SpreadElement", "argument") => {
            rules.argument || rules.assignment || rules.enum_assignment
        }
        ("AssignmentExpression" | "AssignmentPattern", "left" | "right")
        | ("VariableDeclarator", "id" | "init")
        | ("PropertyDefinition" | "AccessorProperty", "key" | "value") => {
            rules.assignment || rules.enum_assignment
        }
        ("Property", "key" | "value") => rules.assignment || rules.enum_assignment,
        ("UpdateExpression", "argument") => rules.enum_assignment,
        ("TSAsExpression" | "TSTypeAssertion", "expression") => rules.enum_assignment,
        ("ArrayExpression", "elements") => rules.enum_assignment,
        ("ReturnStatement", "argument") | ("ArrowFunctionExpression", "body") => {
            rules.returns || rules.enum_assignment
        }
        ("BinaryExpression", "left" | "right") => {
            rules.enum_comparison
                || (rules.enum_assignment && matches!(operator, Some("&" | "|" | "^")))
        }
        ("UnaryExpression", "argument") => {
            (rules.unary_minus && operator == Some("-"))
                || (rules.enum_assignment && operator == Some("~"))
        }
        ("SwitchCase", "test") => rules.enum_comparison,
        ("MemberExpression", "object") => rules.member_access,
        ("MemberExpression", "property") => {
            rules.member_access && parent.field_bool("computed") == Some(true)
        }
        _ => false,
    }
}

/// no-unsafe-enum-assignment reads a type assertion's own type as the
/// receiver of its expression.
pub(super) fn detail_on_self(node: &LintNode, needs: Needs) -> bool {
    needs.detail.enum_assignment
        && matches!(node.kind.as_str(), "TSAsExpression" | "TSTypeAssertion")
}

/// Queries for the listener node itself (call signature, own function returns)
/// and, for a `return`, its enclosing function.
pub(super) fn listener_queries(node: &LintNode, queries: &mut Planner, needs: Needs) {
    if needs.destructuring
        && let Some((sender, pattern)) = destructuring(node)
    {
        let mut targets = Vec::new();
        pattern_paths(pattern, &mut Vec::new(), &mut targets);
        if !targets.is_empty() {
            let mut query = query(sender.range.start, sender.range.end, 0, None);
            query.paths = targets.into_iter().map(|(_, path)| path).collect();
            queries.push(query);
        }
    }
    let contextual = |start: u32, end: u32| {
        let mut query = ExpressionQuery::at(start, end);
        query.contextual = true;
        query
    };
    match node.kind.as_str() {
        "SwitchCase" if needs.switch_cases => {
            if let Some((start, end)) = switch_discriminant(node) {
                let mut query = ExpressionQuery::at(start, end);
                query.detail = true;
                queries.push(query);
            }
        }
        "Property" if needs.property_keys => {
            if let Some(key) = contextual_key(node) {
                queries.push(contextual(key.range.start, key.range.end));
            }
        }
        "ArrayExpression" if needs.literal_members => {
            for element in node.child_list("elements").unwrap_or(&[]) {
                queries.push(contextual(element.range.start, element.range.end));
            }
        }
        "ObjectExpression" if needs.literal_members => {
            queries.push(contextual(node.range.start, node.range.end));
            for value in object_values(node) {
                queries.push(contextual(value.range.start, value.range.end));
            }
        }
        "CallExpression" | "NewExpression" | "TaggedTemplateExpression" => {
            if needs.calls {
                let arguments = call_arguments(node);
                queries.push(query(
                    node.range.start,
                    node.range.end,
                    arguments + SPREAD_SLACK,
                    None,
                ));
            }
            if needs.super_calls
                && let Some((start, end)) = super_call_base(node)
            {
                // The callee is typed from this base, detail included.
                let mut query = ExpressionQuery::at(start, end);
                query.detail = true;
                queries.push(query);
            }
        }
        "ArrowFunctionExpression" if needs.returns && has_expression_body(node) => {
            let contextual = contextual_parent(
                node.field_str("__parentKind"),
                node.field_str("__keyInParent"),
            );
            queries.push(query(node.range.start, node.range.end, 0, Some(contextual)));
        }
        "ReturnStatement" if needs.returns => {
            if let Some(function) = enclosing_function(node) {
                queries.push(query(
                    function.start,
                    function.end,
                    0,
                    Some(function.contextual),
                ));
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

/// Facts by query range. Queries are merged per range, so one fact answers
/// every family that asked about that range.
pub(super) type ListenerFacts<'a> = FxHashMap<(u32, u32), &'a ExpressionFact>;

/// Stamps the checker facts gathered by [`listener_queries`].
pub(super) fn stamp_listener(node: &mut LintNode, facts: &ListenerFacts<'_>) {
    stamp_destructuring(node, facts);
    let (calls, returns, contextuals) = (facts, facts, facts);
    match node.kind.as_str() {
        "SwitchCase" => {
            let detail = switch_discriminant(node)
                .and_then(|range| facts.get(&range))
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
                    if property.kind == "Property"
                        && let Some(value) = property.children.get_mut("value")
                    {
                        stamp_contextual(value, contextuals);
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
            if let Some(base) = super_call_base(node).and_then(|range| facts.get(&range)) {
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

fn stamp_contextual(node: &mut LintNode, contextuals: &ListenerFacts<'_>) {
    if let Some(contextual) = contextuals
        .get(&(node.range.start, node.range.end))
        .and_then(|fact| fact.contextual.clone())
    {
        node.fields.insert("__contextualType".into(), contextual);
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
        call_positions,
        function_returns,
        ..ExpressionQuery::at(start, end)
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

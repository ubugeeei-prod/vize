//! Which checker facts to request for which nodes.
//!
//! Every visited node gets its type text, flags and property names. Every
//! other fact costs extra checker requests, so it is requested only for the
//! positions an enabled rule reads it from: the plan is a function of the
//! enabled rules and of each node's role (parent kind and key), and one query
//! per range carries every family's needs.
mod members;
mod tree;
use super::checker::{self, Needs as CheckerNeeds};
use super::promise::{self, PromiseRules};
use super::type_shape;
use crate::linter::corsa_session::{ComputedMemberQuery, ExpressionQuery, Facets, HeritageQuery};
use corsa::lint::LintNode;
use members::{accessed_name, is_receiver_position, pattern_keys};
use serde_json::Value;
use tree::{TreeNeeds, wants_tree};
use vize_l0::{FxHashMap, FxHashSet, String};

/// What the enabled rules read, by fact family.
#[derive(Clone, Copy, Default)]
pub(in super::super) struct Wants {
    promise: PromiseRules,
    checker: CheckerNeeds,
    tree: TreeNeeds,
    /// `__typeShape` for the no-unnecessary-* rules.
    shapes: bool,
    /// Type predicates for no-unnecessary-condition's `checkTypePredicates`.
    predicates: bool,
    /// Union constituents of union/intersection members.
    type_parts: bool,
    /// The empty-object probe on intersections and generic references.
    empty_object: bool,
    /// Inferred signature/member types for no-unnecessary-type-parameters.
    type_parameters: bool,
}

impl Wants {
    /// `corsa_rules` are the enabled corsa rule names. The other inputs are
    /// whether vize's own `no-generated-empty-object-type` and
    /// `no-floating-promises` run here, and whether no-unnecessary-condition's
    /// `checkTypePredicates` is on.
    pub(in super::super) fn new(
        corsa_rules: &[&str],
        empty_object: bool,
        floating: bool,
        predicates: bool,
    ) -> Self {
        let has = |name: &str| corsa_rules.contains(&name);
        let shapes = [
            "no-unnecessary-condition",
            "no-unnecessary-template-expression",
            "no-unnecessary-type-conversion",
            "no-unnecessary-boolean-literal-compare",
        ]
        .iter()
        .any(|rule| has(rule));
        Self {
            promise: PromiseRules::from_rules(corsa_rules, floating),
            checker: CheckerNeeds::for_rules(corsa_rules),
            tree: TreeNeeds::from_rules(corsa_rules),
            shapes,
            predicates: predicates && has("no-unnecessary-condition"),
            type_parts: has("no-redundant-type-constituents")
                || has("no-duplicate-type-constituents"),
            empty_object,
            type_parameters: has("no-unnecessary-type-parameters"),
        }
    }

    pub(in super::super) fn promise(self) -> PromiseRules {
        self.promise
    }

    pub(in super::super) fn empty_object(self) -> bool {
        self.empty_object
    }
}

/// One query per range: a repeated range merges what it asks for.
#[derive(Default)]
pub(super) struct Planner {
    queries: Vec<ExpressionQuery>,
    seen: FxHashMap<(u32, u32), usize>,
}

impl Planner {
    pub(super) fn push(&mut self, query: ExpressionQuery) {
        if let Some(existing) = self
            .seen
            .get(&(query.start, query.end))
            .and_then(|&index| self.queries.get_mut(index))
        {
            existing.merge(query);
            return;
        }
        self.seen
            .insert((query.start, query.end), self.queries.len());
        self.queries.push(query);
    }
}

/// Builds one query per distinct range for `nodes`, for the enabled rules.
pub(in super::super) fn queries(nodes: &[LintNode], wants: Wants) -> Vec<ExpressionQuery> {
    let mut members = FxHashSet::default();
    if wants.type_parts {
        for node in nodes {
            type_members(node, &mut members);
        }
    }
    let mut plan = Planner::default();
    for node in nodes {
        let role = Role {
            promise: promise::root_position(node),
            ..Role::default()
        };
        collect(node, role, wants, &members, &mut plan);
        checker::listener_queries(node, &mut plan, wants.checker);
    }
    if wants.shapes {
        type_shape::request(nodes, wants.predicates, &mut plan);
    }
    plan.queries
}

/// What a node's parent position asks of it.
#[derive(Clone, Copy, Default)]
struct Role<'a> {
    /// `(parent kind, key)` as the promise rules name positions.
    promise: Option<(&'a str, &'a str)>,
    /// Type tree details (`detail::*`).
    tree: u8,
    /// `__typeDetail`.
    detail: bool,
    /// Awaited-`any` discrimination for `no-unsafe-return`.
    promise_any: bool,
}

fn collect(
    node: &LintNode,
    role: Role<'_>,
    wants: Wants,
    members: &FxHashSet<(u32, u32)>,
    plan: &mut Planner,
) {
    if wants.type_parameters && node.kind == "TSTypeParameterDeclaration" {
        type_parameter_queries(node, plan);
    }
    let rules = wants.promise;
    let mut own = ExpressionQuery::at(node.range.start, node.range.end);
    own.promise = promise_facets(node, role.promise, rules);
    own.contextual_function = rules.misused && is_contextual_position(role.promise);
    if rules.misused {
        own.contextual_methods = method_names(node);
        own.heritage = promise::heritage_query(node);
    }
    own.detail = role.detail || checker::detail_on_self(node, wants.checker);
    own.promise_any = role.promise_any;
    own.tree = role.tree;
    if wants.tree.members && node.kind == "ObjectPattern" {
        own.member_methods = pattern_keys(node);
    }
    own.parts = members.contains(&(node.range.start, node.range.end));
    own.empty_object = wants.empty_object && may_generate_empty_object(node);
    plan.push(own);
    if rules.misused {
        // `implements` clauses are heritage types of their own.
        let names = promise::class_member_names(node);
        for clause in node.child_list("implements").unwrap_or_default() {
            if names.is_empty() {
                break;
            }
            let mut query = ExpressionQuery::at(clause.range.start, clause.range.end);
            query.heritage = Some(HeritageQuery {
                base_types: false,
                members: names.clone(),
            });
            plan.push(query);
        }
    }
    callee_query(node, wants, plan);
    if wants.tree.members {
        member_queries(node, plan);
    }
    let promise_kind = promise::position_kind(node, role.promise);
    let child_role = |key: &'_ str, index: usize| Role {
        promise: None,
        tree: wants_tree(wants.tree, node, key, index),
        detail: checker::detail_at(wants.checker, node, key),
        promise_any: checker::promise_any_at(wants.checker, node, key),
    };
    for (key, child) in &node.children {
        let role = Role {
            promise: Some((promise_kind, key.as_str())),
            ..child_role(key, 0)
        };
        collect(child, role, wants, members, plan);
    }
    for (key, children) in &node.child_lists {
        for (index, child) in children.iter().enumerate() {
            let role = Role {
                promise: Some((promise_kind, key.as_str())),
                ..child_role(key, index)
            };
            collect(child, role, wants, members, plan);
        }
    }
}

include!("queries/type_parameters.rs");

/// The callee (or tag) of a call: its signatures, `Function` signature
/// counts and void-expecting argument positions.
fn callee_query(node: &LintNode, wants: Wants, plan: &mut Planner) {
    let callee = match node.kind.as_str() {
        "CallExpression" | "NewExpression" => node.child("callee"),
        "TaggedTemplateExpression" => node.child("tag"),
        _ => None,
    };
    let Some(callee) = callee else {
        return;
    };
    let mut query = ExpressionQuery::at(callee.range.start, callee.range.end);
    query.signatures = true;
    // no-implied-eval and no-base-to-string skip a callee declared in this
    // file (a local `setTimeout` or `String` shadows the global).
    query.symbols = callee.kind == "Identifier";
    let count = node.child_list("arguments").map_or(0, <[LintNode]>::len);
    if wants.promise.misused && count > 0 && node.kind != "TaggedTemplateExpression" {
        query.void_arguments = Some((node.kind == "NewExpression", count));
    }
    plan.push(query);
}

/// unbound-method reads the accessed property on the receiver's type, and
/// the destructured properties on a pattern's initializer.
fn member_queries(node: &LintNode, plan: &mut Planner) {
    if node.kind == "MemberExpression" && !is_receiver_position(node) {
        if let (Some(object), Some(name)) = (node.child("object"), accessed_name(node)) {
            let mut query = ExpressionQuery::at(object.range.start, object.range.end);
            query.symbols = object.kind == "Identifier";
            query.member_methods = vec![name];
            plan.push(query);
        } else if node.field_bool("computed") == Some(true)
            && let (Some(object), Some(property)) = (node.child("object"), node.child("property"))
        {
            let mut query = ExpressionQuery::at(node.range.start, node.range.end);
            query.computed_member = Some(ComputedMemberQuery {
                object: (object.range.start, object.range.end),
                property: (property.range.start, property.range.end),
            });
            plan.push(query);
        }
    }
    if node.kind == "ObjectPattern"
        && let Some(range) = node.fields.get("__initRange")
        && let (Some(start), Some(end)) = (field_u32(range, "start"), field_u32(range, "end"))
    {
        let mut query = ExpressionQuery::at(start, end);
        query.symbols = node.field_str("__initName").is_some();
        query.member_methods = pattern_keys(node);
        plan.push(query);
    }
}

fn field_u32(value: &Value, key: &str) -> Option<u32> {
    u32::try_from(value.get(key)?.as_u64()?).ok()
}

/// Promise facets at a position. Only function values can be catch handlers.
fn promise_facets(node: &LintNode, position: Option<(&str, &str)>, rules: PromiseRules) -> Facets {
    if !is_value_node(node) {
        return Facets::default();
    }
    let facets = promise::facets(position, rules);
    if matches!(
        node.kind.as_str(),
        "ArrowFunctionExpression" | "FunctionExpression"
    ) {
        facets
    } else {
        Facets(facets.0 & !Facets::CATCH_HANDLER)
    }
}

/// Nodes whose value type a promise rule may read.
fn is_value_node(node: &LintNode) -> bool {
    let kind = node.kind.as_str();
    (kind.ends_with("Expression") && !kind.starts_with("TS"))
        || matches!(
            kind,
            "Identifier"
                | "Literal"
                | "SpreadElement"
                | "TemplateLiteral"
                | "TSAsExpression"
                | "TSSatisfiesExpression"
                | "TSNonNullExpression"
                | "TSTypeAssertion"
        )
}

/// Positions where typescript-eslint reads the checker's contextual type.
fn is_contextual_position(position: Option<(&str, &str)>) -> bool {
    matches!(
        position,
        Some(
            ("Property", "value")
                | ("ReturnStatement", "argument")
                | ("CallExpression" | "NewExpression", "arguments")
        )
    )
}

/// Non-computed method names of an object literal, which typescript-eslint
/// resolves against the literal's contextual type.
fn method_names(node: &LintNode) -> Vec<String> {
    if node.kind != "ObjectExpression" {
        return Vec::new();
    }
    node.child_list("properties")
        .unwrap_or_default()
        .iter()
        .filter(|property| {
            property.kind == "Property"
                && property.field_bool("method") == Some(true)
                && property.field_bool("computed") != Some(true)
        })
        .filter_map(|property| {
            let key = property.child("key")?;
            key.field_str("name")
                .or_else(|| key.field_str("value"))
                .map(String::from)
        })
        .collect()
}

/// Ranges of every union/intersection member under `node`.
fn type_members(node: &LintNode, members: &mut FxHashSet<(u32, u32)>) {
    if matches!(node.kind.as_str(), "TSUnionType" | "TSIntersectionType") {
        for member in node.child_list("types").unwrap_or(&[]) {
            members.insert((member.range.start, member.range.end));
        }
    }
    for child in node.children.values() {
        type_members(child, members);
    }
    for children in node.child_lists.values() {
        for child in children {
            type_members(child, members);
        }
    }
}

/// The nodes `no-generated-empty-object-type` inspects: intersections and
/// type references with type arguments.
fn may_generate_empty_object(node: &LintNode) -> bool {
    node.kind == "TSIntersectionType"
        || (node.kind == "TSTypeReference" && node.child("typeArguments").is_some())
}

//! Host facts the corsa rules do not infer from type text alone.
use super::tree::Collected;

mod enrich;
mod promise;
mod scan;
mod shape;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery, Facets, HeritageQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use vize_l0::{FxHashMap, String as CompactString};

pub(super) fn prepare(source: &str, collected: &mut Collected) {
    let deprecated = scan::deprecated_names(source);
    let functions = scan::local_functions(&collected.program, source);
    let methods = scan::class_methods(&collected.program);
    let instances = scan::class_instances(&collected.program);
    for node in &mut collected.nodes {
        stamp(node, &deprecated, &functions, &methods, &instances);
    }
}

pub(super) use promise::PromiseRules;

/// One checker query per distinct range, with the needs of every tree that
/// contains the node merged.
pub(super) fn queries(nodes: &[LintNode], rules: PromiseRules) -> Vec<ExpressionQuery> {
    let mut queries = Vec::new();
    for node in nodes {
        let position = promise::root_position(node);
        collect_queries(node, position, rules, &mut queries);
    }
    let mut index = FxHashMap::<(u32, u32), usize>::default();
    let mut merged: Vec<ExpressionQuery> = Vec::with_capacity(queries.len());
    for query in queries {
        match index
            .get(&(query.start, query.end))
            .and_then(|at| merged.get_mut(*at))
        {
            Some(existing) => {
                existing.symbols |= query.symbols;
                existing.signatures |= query.signatures;
                existing.promise = Facets(existing.promise.0 | query.promise.0);
                existing.contextual |= query.contextual;
                if existing.void_arguments.is_none() {
                    existing.void_arguments = query.void_arguments;
                }
                if existing.members.is_empty() {
                    existing.members = query.members;
                }
                if existing.heritage.is_none() {
                    existing.heritage = query.heritage;
                }
            }
            None => {
                index.insert((query.start, query.end), merged.len());
                merged.push(query);
            }
        }
    }
    merged
}

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    enrich::apply(source, nodes, program);
    promise::enrich(nodes, program);
}

pub(super) fn apply(nodes: &mut [LintNode], facts: &[ExpressionFact], queries: &[ExpressionQuery]) {
    let mut merged = FxHashMap::<(u32, u32), ExpressionFact>::default();
    for (fact, query) in facts.iter().zip(queries) {
        match merged.get_mut(&(query.start, query.end)) {
            Some(existing) => merge_fact(existing, fact),
            None => {
                merged.insert((query.start, query.end), fact.clone());
            }
        }
    }
    let by_range = merged
        .iter()
        .map(|(range, fact)| (*range, fact))
        .collect::<FxHashMap<_, _>>();
    for node in nodes {
        apply_node(node, &by_range);
    }
}

fn stamp(
    node: &mut LintNode,
    deprecated: &HashSet<String>,
    functions: &HashMap<String, scan::LocalFn>,
    methods: &HashMap<String, HashSet<String>>,
    instances: &HashMap<String, String>,
) {
    if node.kind == "Identifier"
        && node.field_bool("__declarationName") != Some(true)
        && node
            .field_str("name")
            .is_some_and(|name| deprecated.contains(name))
    {
        node.fields.insert(
            "__symbolFacts".into(),
            json!({ "deprecated": true, "deprecationReason": "" }),
        );
    }
    if node.kind == "CallExpression" {
        stamp_call(node, functions);
    }
    if node.kind == "MemberExpression" {
        stamp_member(node, methods, instances);
    }
    for child in node.children.values_mut() {
        stamp(child, deprecated, functions, methods, instances);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, deprecated, functions, methods, instances);
        }
    }
}

fn stamp_call(node: &mut LintNode, functions: &HashMap<String, scan::LocalFn>) {
    let property = node
        .child("callee")
        .and_then(|callee| callee.child("property"))
        .and_then(|property| property.field_str("name"));
    if property == Some("forEach") {
        node.fields.insert(
            "__expectedArgumentReturnTypeTexts".into(),
            json!([["void"]]),
        );
    }
    let Some(name) = node
        .child("callee")
        .and_then(|callee| callee.field_str("name"))
        .map(str::to_owned)
    else {
        return;
    };
    let Some(function) = functions.get(&name) else {
        return;
    };
    let arguments = type_argument_nodes(node);
    if arguments.is_empty() || function.params.is_empty() {
        return;
    }
    let last = arguments.len() - 1;
    let ranges = arguments
        .iter()
        .map(|argument| (argument.range.start, argument.range.end))
        .collect::<Vec<_>>();
    let argument_text = arguments
        .get(last)
        .and_then(|argument| argument.text.clone());
    let range_values = ranges
        .iter()
        .map(|(start, end)| json!({ "start": start, "end": end }))
        .collect::<Vec<_>>();
    node.fields
        .insert("__typeArgumentRanges".into(), Value::Array(range_values));
    node.fields
        .insert("__typeParameterCount".into(), json!(function.params.len()));
    let default_text = function
        .params
        .get(last)
        .and_then(|(_, default)| default.clone());
    node.fields.insert(
        "__lastTypeParameterHasDefault".into(),
        json!(default_text.is_some()),
    );
    node.fields.insert(
        "__lastTypeArgumentEqualsDefault".into(),
        json!(default_text.is_some() && default_text == argument_text),
    );
}

fn type_argument_nodes(node: &LintNode) -> &[LintNode] {
    node.child("typeArguments")
        .or_else(|| node.child("typeParameters"))
        .and_then(|list| list.child_list("params"))
        .unwrap_or(&[])
}

fn stamp_member(
    node: &mut LintNode,
    methods: &HashMap<String, HashSet<String>>,
    instances: &HashMap<String, String>,
) {
    if node.field_str("__parentKind") == Some("CallExpression") {
        node.fields.insert("__safeUse".into(), json!(true));
        return;
    }
    let Some(object) = node
        .child("object")
        .and_then(|object| object.field_str("name"))
    else {
        return;
    };
    let Some(class_name) = instances.get(object) else {
        return;
    };
    let Some(property) = node
        .child("property")
        .and_then(|property| property.field_str("name"))
    else {
        return;
    };
    if methods
        .get(class_name)
        .is_some_and(|names| names.contains(property))
    {
        node.fields.insert(
            "__unboundMethodInfo".into(),
            json!({ "isMethod": true, "firstParamIsThis": false, "thisArgIsVoid": false, "isStatic": false }),
        );
    }
}

fn collect_queries(
    node: &LintNode,
    position: Option<(&str, &str)>,
    rules: PromiseRules,
    queries: &mut Vec<ExpressionQuery>,
) {
    let mut facets = if is_value_node(node) {
        promise::facets(position, rules)
    } else {
        Facets::default()
    };
    if !matches!(
        node.kind.as_str(),
        "ArrowFunctionExpression" | "FunctionExpression"
    ) {
        facets = Facets(facets.0 & !Facets::CATCH_HANDLER);
    }
    queries.push(ExpressionQuery {
        start: node.range.start,
        end: node.range.end,
        symbols: node.kind == "Identifier",
        signatures: false,
        promise: facets,
        contextual: rules.misused && is_contextual_position(position),
        void_arguments: None,
        members: if rules.misused {
            method_names(node)
        } else {
            Vec::new()
        },
        heritage: rules
            .misused
            .then(|| promise::heritage_query(node))
            .flatten(),
    });
    if rules.misused {
        // `implements` clauses are heritage types of their own.
        for clause in node.child_list("implements").unwrap_or_default() {
            let members = promise::class_member_names(node);
            if members.is_empty() {
                continue;
            }
            queries.push(ExpressionQuery {
                start: clause.range.start,
                end: clause.range.end,
                symbols: false,
                signatures: false,
                promise: Facets::default(),
                contextual: false,
                void_arguments: None,
                members: Vec::new(),
                heritage: Some(HeritageQuery {
                    base_types: false,
                    members,
                }),
            });
        }
    }
    if (node.kind == "CallExpression" || node.kind == "NewExpression")
        && let Some(callee) = node.child("callee")
    {
        let count = node.child_list("arguments").map_or(0, <[LintNode]>::len);
        queries.push(ExpressionQuery {
            start: callee.range.start,
            end: callee.range.end,
            symbols: false,
            signatures: node.kind == "CallExpression",
            promise: Facets::default(),
            contextual: false,
            void_arguments: (rules.misused && count > 0)
                .then_some((node.kind == "NewExpression", count)),
            members: Vec::new(),
            heritage: None,
        });
    }
    // Aggregator arrays and promise-method receivers are positions of their own.
    let kind = if node.kind == "ArrayExpression"
        && matches!(position, Some(("CallExpression", "arguments")))
    {
        "ArgumentArray"
    } else if node.kind == "MemberExpression"
        && matches!(position, Some(("CallExpression", "callee")))
        && promise::is_promise_method(node)
    {
        "PromiseMethodCallee"
    } else {
        node.kind.as_str()
    };
    for (child_key, child) in &node.children {
        collect_queries(child, Some((kind, child_key)), rules, queries);
    }
    for (child_key, children) in &node.child_lists {
        for child in children {
            collect_queries(child, Some((kind, child_key)), rules, queries);
        }
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
fn method_names(node: &LintNode) -> Vec<CompactString> {
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
                .map(CompactString::from)
        })
        .collect()
}

fn merge_fact(existing: &mut ExpressionFact, other: &ExpressionFact) {
    if existing.promise.is_none() {
        existing.promise = other.promise;
    }
    if existing.contextual_void.is_none() {
        existing.contextual_void = other.contextual_void;
        existing.contextual_thenable = other.contextual_thenable;
    }
    if existing.contextual_members.is_empty() {
        existing
            .contextual_members
            .clone_from(&other.contextual_members);
    }
    if existing.heritage_voids.is_empty() {
        existing.heritage_voids.clone_from(&other.heritage_voids);
    }
    if existing.void_arguments.is_none() {
        existing.void_arguments.clone_from(&other.void_arguments);
    }
    if existing.parameter_type_texts.is_empty() {
        existing
            .parameter_type_texts
            .clone_from(&other.parameter_type_texts);
    }
    if !existing.deprecated && other.deprecated {
        existing.deprecated = true;
        existing
            .deprecation_reason
            .clone_from(&other.deprecation_reason);
    }
}

fn apply_node(node: &mut LintNode, facts: &FxHashMap<(u32, u32), &ExpressionFact>) {
    if let Some(fact) = facts.get(&(node.range.start, node.range.end)) {
        node.type_texts.clone_from(&fact.type_texts);
        node.property_names.clone_from(&fact.property_names);
        promise::apply(node, fact);
        if fact.deprecated && node.field_bool("__declarationName") != Some(true) {
            node.fields.insert(
                "__symbolFacts".into(),
                json!({
                    "deprecated": true,
                    "deprecationReason": fact.deprecation_reason,
                }),
            );
        }
    }
    if (node.kind == "CallExpression" || node.kind == "NewExpression")
        && let Some(callee) = node.child("callee")
        && let Some(fact) = facts.get(&(callee.range.start, callee.range.end))
    {
        promise::apply_call(node, fact);
        if node.kind == "CallExpression" && !fact.parameter_type_texts.is_empty() {
            node.fields.insert(
                "__expectedArgumentTypeTexts".into(),
                json!(fact.parameter_type_texts),
            );
        }
    }
    let keys = node.children.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        if let Some(child) = node.children.get_mut(&key) {
            apply_node(child, facts);
        }
    }
    let lists = node.child_lists.keys().cloned().collect::<Vec<_>>();
    for key in lists {
        if let Some(children) = node.child_lists.get_mut(&key) {
            for child in children {
                apply_node(child, facts);
            }
        }
    }
}

//! Checker promise facts stamped onto lint nodes.
use crate::linter::corsa_session::{ExpressionFact, Facets, HeritageQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::String as CompactString;

/// The enabled rules that read promise facts.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::linter::native_type_aware::typescript_eslint) struct PromiseRules {
    pub await_thenable: bool,
    pub return_await: bool,
    pub misused: bool,
    pub floating: bool,
    pub reject_errors: bool,
    pub throw_error: bool,
    pub require_await: bool,
    pub catch_callback: bool,
}

impl PromiseRules {
    pub(in crate::linter::native_type_aware::typescript_eslint) fn from_rules(
        rules: &[&str],
        floating: bool,
    ) -> Self {
        Self {
            await_thenable: rules.contains(&"await-thenable"),
            return_await: rules.contains(&"return-await"),
            misused: rules.contains(&"no-misused-promises"),
            floating,
            reject_errors: rules.contains(&"prefer-promise-reject-errors"),
            throw_error: rules.contains(&"only-throw-error"),
            require_await: rules.contains(&"require-await"),
            catch_callback: rules.contains(&"use-unknown-in-catch-callback-variable"),
        }
    }

    pub(in crate::linter::native_type_aware::typescript_eslint) const fn any(self) -> bool {
        self.await_thenable
            || self.return_await
            || self.misused
            || self.floating
            || self.reject_errors
            || self.throw_error
            || self.require_await
            || self.catch_callback
    }
}

/// `(parent kind, key in parent)` of a listener root, from its ancestor chain.
pub(super) fn root_position(node: &LintNode) -> Option<(&str, &str)> {
    let parent = node.fields.get("__ancestorChain")?.as_array()?.first()?;
    Some((
        parent.get("kind")?.as_str()?,
        parent.get("childKey")?.as_str()?,
    ))
}

const CLASS_LIKE: &[&str] = &[
    "ClassDeclaration",
    "ClassExpression",
    "TSInterfaceDeclaration",
];

/// The heritage lookup of a class (its `extends`) or an interface.
pub(super) fn heritage_query(node: &LintNode) -> Option<HeritageQuery> {
    if !CLASS_LIKE.contains(&node.kind.as_str()) {
        return None;
    }
    let has_extends = node.child("superClass").is_some()
        || node
            .child_list("extends")
            .is_some_and(|clauses| !clauses.is_empty());
    let members = class_member_names(node);
    (has_extends && !members.is_empty()).then_some(HeritageQuery {
        base_types: true,
        members,
    })
}

fn class_members(node: &LintNode) -> &[LintNode] {
    node.child("body")
        .and_then(|body| body.child_list("body"))
        .unwrap_or_default()
}

/// The non-static, statically named members typescript-eslint checks.
pub(super) fn class_member_names(node: &LintNode) -> Vec<CompactString> {
    class_members(node).iter().filter_map(member_name).collect()
}

fn member_name(member: &LintNode) -> Option<CompactString> {
    if member.field_bool("static") == Some(true) || member.field_bool("computed") == Some(true) {
        return None;
    }
    let key = member.child("key")?;
    match key.kind.as_str() {
        "Identifier" => key.field_str("name").map(CompactString::from),
        "Literal" => key.field_str("value").map(CompactString::from),
        _ => None,
    }
}

/// Members that return a thenable where a heritage type expects `void`.
fn stamp_heritage(node: &mut LintNode) {
    if !CLASS_LIKE.contains(&node.kind.as_str()) {
        return;
    }
    let mut voids = Vec::new();
    let mut gather = |source: &LintNode| {
        for entry in source
            .fields
            .get("__heritageVoidMembers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let (Some(member), Some(heritage)) = (
                entry.get(0).and_then(Value::as_str),
                entry.get(1).and_then(Value::as_str),
            ) {
                voids.push((CompactString::from(member), CompactString::from(heritage)));
            }
        }
    };
    gather(node);
    for clause in node.child_list("implements").unwrap_or_default() {
        gather(clause);
    }
    if voids.is_empty() {
        return;
    }
    let Some(members) = node
        .children
        .get_mut("body")
        .and_then(|body| body.child_lists.get_mut("body"))
    else {
        return;
    };
    for member in members {
        let Some(name) = member_name(member) else {
            continue;
        };
        let returns_thenable = member
            .child("key")
            .and_then(|key| key.fields.get("__promiseFacts"))
            .and_then(|facts| facts.get("returnsThenable"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !returns_thenable {
            continue;
        }
        let names = voids
            .iter()
            .filter(|(member, _)| *member == name)
            .map(|(_, heritage)| json!(heritage.as_str()))
            .collect::<Vec<_>>();
        if !names.is_empty() {
            member
                .fields
                .insert("__voidReturnHeritageTypeNames".into(), Value::Array(names));
        }
    }
}

/// A callee `x.then` / `x.catch` / `x.finally` / `x.reject` / `x.all` ...
/// whose object a promise rule inspects. Computed keys may name one too.
pub(super) fn is_promise_method(member: &LintNode) -> bool {
    if member.field_bool("computed") == Some(true) {
        return true;
    }
    member
        .child("property")
        .and_then(|property| property.field_str("name"))
        .is_some_and(|name| {
            matches!(
                name,
                "then"
                    | "catch"
                    | "finally"
                    | "reject"
                    | "all"
                    | "allSettled"
                    | "any"
                    | "race"
                    | "every"
                    | "filter"
                    | "find"
                    | "findIndex"
                    | "findLast"
                    | "findLastIndex"
                    | "some"
            )
        })
}

/// The promise facts the enabled rules read at a syntax position, mirroring
/// where each typescript-eslint rule asks the checker.
pub(super) fn facets(position: Option<(&str, &str)>, rules: PromiseRules) -> Facets {
    const FLOATING: u16 = Facets::PROMISE | Facets::PROMISE_ARRAY;
    // `checksConditionals` with every `flagUnions` mode.
    const CONDITIONAL: u16 = Facets::ALWAYS_THENABLE | Facets::THENABLE | Facets::MATCHING_UNION;
    let Some((parent, key)) = position else {
        return Facets::default();
    };
    let mut wanted = 0;
    let mut want = |enabled: bool, facets: u16| {
        if enabled {
            wanted |= facets;
        }
    };
    let awaits = rules.await_thenable || rules.return_await;
    match (parent, key) {
        ("AwaitExpression", "argument") => want(awaits, Facets::AWAITABLE),
        ("ReturnStatement", "argument") => {
            want(rules.require_await, Facets::THENABLE);
            want(rules.return_await, Facets::AWAITABLE);
            want(rules.misused, Facets::RETURNS_THENABLE);
        }
        ("ArrowFunctionExpression", "body") => {
            want(rules.return_await, Facets::AWAITABLE);
            want(rules.require_await, Facets::THENABLE);
        }
        ("YieldExpression", "argument") => {
            want(
                rules.require_await,
                Facets::THENABLE | Facets::ASYNC_ITERABLE,
            );
        }
        ("ConditionalExpression", "consequent" | "alternate") => {
            want(rules.catch_callback, Facets::CATCH_HANDLER);
            want(rules.return_await, Facets::AWAITABLE);
            want(rules.floating, FLOATING);
        }
        (
            "IfStatement"
            | "WhileStatement"
            | "DoWhileStatement"
            | "ForStatement"
            | "ConditionalExpression",
            "test",
        ) => want(rules.misused, CONDITIONAL),
        ("LogicalExpression", "left" | "right") | ("UnaryExpression", "argument") => {
            want(rules.catch_callback, Facets::CATCH_HANDLER);
            want(rules.misused, CONDITIONAL);
            want(rules.floating, FLOATING);
        }
        ("SpreadElement", "argument") => want(rules.misused, Facets::THENABLE),
        ("ArgumentArray", "elements") => want(rules.await_thenable, Facets::AGGREGATOR),
        ("ForOfStatement", "right") => want(rules.await_thenable, Facets::ASYNC_ITERABLE),
        ("CallExpression" | "NewExpression", "arguments") => {
            want(rules.catch_callback, Facets::CATCH_HANDLER);
            want(rules.await_thenable, Facets::AGGREGATOR);
            want(rules.misused, Facets::RETURNS_THENABLE);
            want(rules.floating, Facets::CALLABLE);
            want(rules.reject_errors, Facets::ERROR);
        }
        ("VariableDeclarator", "init")
        | ("AssignmentExpression", "right")
        | ("Property", "value") => want(rules.misused, Facets::RETURNS_THENABLE),
        (
            "MethodDefinition"
            | "PropertyDefinition"
            | "TSAbstractMethodDefinition"
            | "TSAbstractPropertyDefinition"
            | "AccessorProperty"
            | "TSMethodSignature"
            | "TSPropertySignature",
            "key",
        ) => want(rules.misused, Facets::RETURNS_THENABLE),
        ("VariableDeclarator", "id") | ("AssignmentExpression", "left") => {
            want(rules.misused, Facets::VOID_FUNCTION);
        }
        ("PromiseMethodCallee", "object") => {
            want(rules.misused, Facets::ARRAY_OR_TUPLE);
            want(rules.catch_callback || rules.throw_error, Facets::THENABLE);
            want(
                rules.await_thenable || rules.reject_errors,
                Facets::PROMISE_CONSTRUCTOR,
            );
            want(rules.misused || rules.reject_errors, Facets::PROMISE);
            want(rules.floating, FLOATING);
        }
        ("NewExpression", "callee") => want(rules.reject_errors, Facets::PROMISE_CONSTRUCTOR),
        ("ExpressionStatement", "expression") | ("SequenceExpression", "expressions") => {
            want(rules.catch_callback, Facets::CATCH_HANDLER);
            want(rules.floating, FLOATING);
        }
        ("ThrowStatement", "argument") => want(rules.throw_error, Facets::ERROR),
        // A chain keeps the needs of the position it fills.
        ("ChainExpression", "expression") => {
            want(awaits || rules.return_await, Facets::AWAITABLE);
            want(
                rules.misused,
                Facets::ALWAYS_THENABLE | Facets::RETURNS_THENABLE,
            );
            want(rules.floating, FLOATING);
            want(
                rules.reject_errors,
                Facets::ERROR | Facets::PROMISE_CONSTRUCTOR,
            );
            want(rules.throw_error, Facets::ERROR);
        }
        _ => {}
    }
    Facets(wanted)
}

pub(super) fn apply(node: &mut LintNode, fact: &ExpressionFact) {
    if let Some(promise) = fact.promise {
        let mut facts = serde_json::Map::new();
        facts.insert("flags".into(), json!(promise.flags));
        let mut put = |key: &str, value: Option<bool>| {
            if let Some(value) = value {
                facts.insert(key.into(), json!(value));
            }
        };
        put("thenable", promise.thenable);
        put("alwaysThenable", promise.always_thenable);
        put("returnsThenable", promise.returns_thenable);
        put("voidFunction", promise.void_function);
        put("promise", promise.promise);
        put("promiseArray", promise.promise_array);
        put("callable", promise.callable);
        put("error", promise.error);
        put("promiseConstructor", promise.promise_constructor);
        put("nonAwaitable", promise.non_awaitable);
        put("invalidAggregatorInput", promise.invalid_aggregator_input);
        put("maybeAwaitableValue", promise.maybe_awaitable_value);
        put("asyncIterable", promise.async_iterable);
        put("asyncDisposable", promise.async_disposable);
        put("matchingPromiseUnion", promise.matching_promise_union);
        put("flaggableHandler", promise.flaggable_handler);
        put("arrayOrTuple", promise.array_or_tuple);
        if let Some(awaitable) = promise.awaitable {
            facts.insert("awaitable".into(), json!(awaitable.as_str()));
            node.fields
                .insert("__awaitableCertainty".into(), json!(awaitable.as_str()));
        }
        if let Some(always) = promise.always_thenable {
            node.fields
                .insert("__allTypePartsThenable".into(), json!(always));
        }
        node.fields
            .insert("__promiseFacts".into(), Value::Object(facts));
    }
    if let Some(void) = fact.contextual_void {
        node.fields
            .insert("__contextualVoidFunction".into(), json!(void));
    }
    if let Some(thenable) = fact.contextual_thenable {
        node.fields
            .insert("__contextualThenableFunction".into(), json!(thenable));
    }
    if !fact.heritage_voids.is_empty() {
        let voids = fact
            .heritage_voids
            .iter()
            .map(|(member, heritage)| json!([member.as_str(), heritage.as_str()]))
            .collect::<Vec<_>>();
        node.fields
            .insert("__heritageVoidMembers".into(), Value::Array(voids));
    }
    if !fact.contextual_members.is_empty() {
        let members = fact
            .contextual_members
            .iter()
            .map(|(name, void)| (name.as_str().into(), json!(void)))
            .collect::<serde_json::Map<_, _>>();
        node.fields.insert(
            "__contextualMemberVoidFunctions".into(),
            Value::Object(members),
        );
    }
}

pub(super) fn apply_call(node: &mut LintNode, fact: &ExpressionFact) {
    if let Some((void, thenable)) = &fact.void_arguments {
        node.fields
            .insert("__voidArgumentIndices".into(), json!(void));
        node.fields
            .insert("__thenableArgumentIndices".into(), json!(thenable));
    }
}

/// Syntax facts the promise rules need beyond the checker: the error-handling
/// context of each `return` and the reject calls of Promise executors.
pub(super) fn enrich(nodes: &mut [LintNode], program: &Value) {
    let mut rejects = Vec::new();
    let mut objects = Vec::new();
    for node in nodes.iter() {
        collect_executor_rejects(node, &mut rejects);
        collect_member_facts(node, &mut objects);
    }
    for node in nodes {
        let parent = parent_range(node);
        stamp(node, program, &rejects, &objects, parent);
    }
}

type MemberFacts = ((u32, u32), Value);

fn collect_member_facts(node: &LintNode, out: &mut Vec<MemberFacts>) {
    if let Some(members) = node
        .fields
        .get("__contextualMemberVoidFunctions")
        .filter(|members| members.is_object())
    {
        out.push(((node.range.start, node.range.end), members.clone()));
    }
    for child in node.children.values() {
        collect_member_facts(child, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_member_facts(child, out);
        }
    }
}

/// Range of a root node's parent, from its ancestor facts.
fn parent_range(node: &LintNode) -> Option<(u32, u32)> {
    let parent = node.fields.get("__ancestorFacts")?.as_array()?.last()?;
    let start = u32::try_from(parent.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(parent.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

/// Object literal methods take `__contextualVoidFunction` from their object's
/// contextual property type.
fn stamp_method(node: &mut LintNode, objects: &[MemberFacts], parent: Option<(u32, u32)>) {
    if node.kind != "Property" || node.field_bool("method") != Some(true) {
        return;
    }
    let Some(parent) = parent else {
        return;
    };
    let Some(name) = node
        .child("key")
        .and_then(|key| key.field_str("name").or_else(|| key.field_str("value")))
        .map(CompactString::from)
    else {
        return;
    };
    let void = objects
        .iter()
        .find(|(range, _)| *range == parent)
        .and_then(|(_, members)| members.get(name.as_str()))
        .and_then(Value::as_bool);
    if let (Some(void), Some(value)) = (void, node.children.get_mut("value")) {
        value
            .fields
            .insert("__contextualVoidFunction".into(), json!(void));
    }
}

fn stamp(
    node: &mut LintNode,
    program: &Value,
    rejects: &[(u32, u32)],
    objects: &[MemberFacts],
    parent: Option<(u32, u32)>,
) {
    stamp_method(node, objects, parent);
    stamp_heritage(node);
    if node.kind == "ReturnStatement"
        && !node.fields.contains_key("__returnAwaitRequiresAwait")
        && let Some(affects) = return_affects_error_handling(program, node)
    {
        node.fields
            .insert("__returnAwaitRequiresAwait".into(), json!(affects));
    }
    if node.kind == "CallExpression" && rejects.contains(&(node.range.start, node.range.end)) {
        node.fields
            .insert("__promiseExecutorRejectCall".into(), json!(true));
    }
    let range = Some((node.range.start, node.range.end));
    for child in node.children.values_mut() {
        stamp(child, program, rejects, objects, range);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, program, rejects, objects, range);
        }
    }
}

/// `new P((resolve, reject) => ...)` where `P` is a Promise constructor: every
/// call whose callee is the `reject` parameter.
fn collect_executor_rejects(node: &LintNode, out: &mut Vec<(u32, u32)>) {
    if node.kind == "NewExpression" && is_promise_constructor(node) {
        let executor = node
            .child_list("arguments")
            .and_then(|arguments| arguments.first())
            .filter(|executor| {
                matches!(
                    executor.kind.as_str(),
                    "ArrowFunctionExpression" | "FunctionExpression"
                )
            });
        if let Some(executor) = executor {
            let reject = executor
                .child_list("params")
                .and_then(|params| params.get(1))
                .filter(|param| param.kind == "Identifier")
                .and_then(|param| param.field_str("name"));
            if let Some(reject) = reject {
                for param in executor.child_list("params").unwrap_or_default() {
                    collect_calls_to(param, reject, out);
                }
                if let Some(body) = executor.child("body") {
                    collect_calls_to(body, reject, out);
                }
            }
        }
    }
    for child in node.children.values() {
        collect_executor_rejects(child, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_executor_rejects(child, out);
        }
    }
}

fn is_promise_constructor(node: &LintNode) -> bool {
    let mut callee = node.child("callee");
    while let Some(current) = callee.filter(|callee| callee.kind == "ChainExpression") {
        callee = current.child("expression");
    }
    callee
        .and_then(|callee| callee.fields.get("__promiseFacts"))
        .and_then(|facts| facts.get("promiseConstructor"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Calls of `name` that still refer to the executor parameter.
fn collect_calls_to(node: &LintNode, name: &str, out: &mut Vec<(u32, u32)>) {
    if is_function(node) && binds_parameter(node, name) {
        return;
    }
    if node.kind == "BlockStatement" && declares(node, name) {
        return;
    }
    if node.kind == "CallExpression"
        && node.child("callee").is_some_and(|callee| {
            callee.kind == "Identifier" && callee.field_str("name") == Some(name)
        })
    {
        out.push((node.range.start, node.range.end));
    }
    for child in node.children.values() {
        collect_calls_to(child, name, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_calls_to(child, name, out);
        }
    }
}

fn is_function(node: &LintNode) -> bool {
    matches!(
        node.kind.as_str(),
        "ArrowFunctionExpression" | "FunctionExpression" | "FunctionDeclaration"
    )
}

fn binds_parameter(function: &LintNode, name: &str) -> bool {
    function
        .child_list("params")
        .unwrap_or_default()
        .iter()
        .any(|param| binds(param, name))
        || (function.kind != "ArrowFunctionExpression" && name == "arguments")
}

fn binds(pattern: &LintNode, name: &str) -> bool {
    match pattern.kind.as_str() {
        "Identifier" => pattern.field_str("name") == Some(name),
        "AssignmentPattern" => pattern.child("left").is_some_and(|left| binds(left, name)),
        "RestElement" => pattern
            .child("argument")
            .is_some_and(|argument| binds(argument, name)),
        "ArrayPattern" => pattern
            .child_list("elements")
            .unwrap_or_default()
            .iter()
            .any(|element| binds(element, name)),
        "ObjectPattern" => pattern
            .child_list("properties")
            .unwrap_or_default()
            .iter()
            .any(|property| {
                property
                    .child("value")
                    .or_else(|| property.child("argument"))
                    .is_some_and(|value| binds(value, name))
            }),
        _ => false,
    }
}

fn declares(block: &LintNode, name: &str) -> bool {
    block
        .child_list("body")
        .unwrap_or_default()
        .iter()
        .any(|statement| match statement.kind.as_str() {
            "VariableDeclaration" => statement
                .child_list("declarations")
                .unwrap_or_default()
                .iter()
                .any(|declarator| declarator.child("id").is_some_and(|id| binds(id, name))),
            "FunctionDeclaration" | "ClassDeclaration" => statement
                .child("id")
                .is_some_and(|id| id.field_str("name") == Some(name)),
            _ => false,
        })
}

/// return-await's `affectsExplicitErrorHandling || affectsExplicitResourceManagement`.
fn return_affects_error_handling(program: &Value, node: &LintNode) -> Option<bool> {
    let mut path = Vec::new();
    if !find_path(program, None, node.range.start, node.range.end, &mut path) {
        return None;
    }
    let expression_start = node
        .child("argument")
        .map_or(node.range.start, |argument| argument.range.start);
    // `path` ends at the return statement; walk its ancestors outward up to
    // the enclosing function. A try block always affects control flow, a
    // catch clause only when a finally block follows; otherwise the search
    // continues past the try statement.
    for index in (0..path.len().saturating_sub(1)).rev() {
        let (ancestor, _) = path.get(index)?;
        let (_, child_key) = path.get(index + 1)?;
        let kind = ancestor
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if matches!(
            kind,
            "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
        ) {
            break;
        }
        if has_using_before(ancestor, expression_start) {
            return Some(true);
        }
        if kind == "TryStatement" {
            let has_finalizer = ancestor
                .get("finalizer")
                .is_some_and(|finalizer| !finalizer.is_null());
            match *child_key {
                Some("block") => return Some(true),
                Some("handler") if has_finalizer => return Some(true),
                _ => {}
            }
        }
    }
    Some(false)
}

fn has_using_before(block: &Value, before: u32) -> bool {
    let statements = block
        .get("body")
        .and_then(Value::as_array)
        .or_else(|| block.get("consequent").and_then(Value::as_array));
    statements.into_iter().flatten().any(|statement| {
        statement.get("type").and_then(Value::as_str) == Some("VariableDeclaration")
            && matches!(
                statement.get("kind").and_then(Value::as_str),
                Some("using" | "await using")
            )
            && statement
                .get("declarations")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|declarator| {
                    declarator
                        .get("end")
                        .and_then(Value::as_u64)
                        .is_some_and(|end| end < u64::from(before))
                })
    })
}

/// Path from the root to the node with `start..end` and a `ReturnStatement`
/// type, as `(node, key in parent)` pairs.
fn find_path<'a>(
    value: &'a Value,
    key: Option<&'a str>,
    start: u32,
    end: u32,
    path: &mut Vec<(&'a Value, Option<&'a str>)>,
) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let node_start = object.get("start").and_then(Value::as_u64);
    let node_end = object.get("end").and_then(Value::as_u64);
    if let (Some(node_start), Some(node_end)) = (node_start, node_end)
        && (node_start > u64::from(start) || node_end < u64::from(end))
    {
        return false;
    }
    path.push((value, key));
    if object.get("type").and_then(Value::as_str) == Some("ReturnStatement")
        && node_start == Some(u64::from(start))
        && node_end == Some(u64::from(end))
    {
        return true;
    }
    for (child_key, child) in object {
        let found = match child {
            Value::Object(_) => find_path(child, Some(child_key.as_str()), start, end, path),
            Value::Array(items) => items
                .iter()
                .any(|item| find_path(item, Some(child_key.as_str()), start, end, path)),
            _ => false,
        };
        if found {
            return true;
        }
    }
    path.pop();
    false
}

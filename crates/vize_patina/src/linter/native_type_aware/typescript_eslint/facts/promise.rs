//! Checker promise facts stamped onto lint nodes.
mod enrich;
use crate::linter::corsa_session::{ExpressionFact, Facets, HeritageQuery};
use corsa::lint::LintNode;
pub(super) use enrich::enrich;
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

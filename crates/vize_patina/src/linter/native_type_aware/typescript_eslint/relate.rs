//! Checker relations for the corsa rules whose verdict needs more than
//! rendered type text: type identity, union-part flags, base constraints,
//! contextual types, assignability and resolved signatures.
mod asks;
mod binding;

use crate::linter::corsa_session::{RelationReply, RelationRequest, Span, type_flags};
use corsa::lint::LintNode;
use oxc_ast::ast::Program;
use serde_json::{Value, json};
use vize_l0::FxHashMap;

/// A host node a reply is written back to: `(kind, start, end)` in script
/// coordinates.
type Key = (&'static str, u32, u32);

/// Facts the host decides without the checker, keyed like the replies.
#[derive(Default)]
pub(super) struct Pending {
    request: RelationRequest,
    assertions: Vec<AssertionTarget>,
    non_null: Vec<NonNullTarget>,
    /// Nodes whose constrained array-likeness a rule reads off `__type`.
    array_like: Vec<Key>,
    deprecations: Vec<Key>,
    defaults: Vec<Key>,
    returns: Vec<Key>,
    /// Assignability replies and the field each is written to.
    assignable: Vec<(Key, &'static str)>,
    /// Nodes whose facts are known without asking the checker.
    fixed: Vec<(Key, &'static str, Value)>,
}

struct AssertionTarget {
    key: Key,
    narrowed_declaration: bool,
}

struct NonNullTarget {
    key: Key,
    used_before_assigned: bool,
}

impl Pending {
    pub(super) fn request(&self) -> &RelationRequest {
        &self.request
    }

    /// Script ranges whose `__type` needs `detail::ARRAY_LIKE`.
    pub(super) fn array_like_ranges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.array_like.iter().map(|&(_, start, end)| (start, end))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.fixed.is_empty()
            && self.request.assertions.is_empty()
            && self.request.non_null.is_empty()
            && self.array_like.is_empty()
            && self.request.deprecations.is_empty()
            && self.request.defaults.is_empty()
            && self.request.returns.is_empty()
            && self.request.assignable.is_empty()
    }
}

/// Build the checker asks for the rules in `rules`.
/// `ast` is the script's own parse (the one the listener tree serializes),
/// for the scope questions the ESTree JSON cannot answer.
pub(super) fn plan(
    ast: &Program<'_>,
    program: &Value,
    rules: &[&str],
    map: &dyn Fn(Span) -> Option<Span>,
) -> Pending {
    let mut pending = Pending::default();
    asks::collect(ast, program, rules, map, &mut pending);
    pending
}

/// Write `reply` (or, when the checker failed, the facts that keep the rules
/// silent) into every matching host node.
pub(super) fn apply(nodes: &mut [LintNode], pending: &Pending, reply: Option<&RelationReply>) {
    let mut facts: Facts = FxHashMap::default();
    let mut push = |(kind, start, end): Key, field: &'static str, value: Value| {
        facts
            .entry((start, end))
            .or_default()
            .push((kind, field, value));
    };
    for (key, field, value) in &pending.fixed {
        push(*key, field, value.clone());
    }
    for (index, target) in pending.assertions.iter().enumerate() {
        let answer = reply.and_then(|reply| reply.assertions.get(index));
        push(
            target.key,
            "__implicitlyNarrowedLiteralDeclaration",
            json!(target.narrowed_declaration),
        );
        push(
            target.key,
            "__typeIsUnchanged",
            json!(answer.is_some_and(|answer| answer.unchanged)),
        );
        push(
            target.key,
            "__castTypeIsLiteral",
            json!(answer.is_some_and(|answer| answer.cast_is_literal)),
        );
        push(
            target.key,
            "__doubleAssertionUnchanged",
            json!(answer.is_some_and(|answer| answer.double_unchanged)),
        );
        push(
            target.key,
            "__contextuallyUnnecessary",
            json!(answer.is_some_and(|answer| answer.contextually_unnecessary)),
        );
        push(
            target.key,
            "__doubleAssertionContextual",
            json!(answer.is_some_and(|answer| answer.double_contextual)),
        );
        if answer.is_none() {
            push(target.key, "__typeFactsMissing", json!(true));
        }
    }
    for (index, target) in pending.non_null.iter().enumerate() {
        let Some(answer) = reply
            .and_then(|reply| reply.non_null.get(index))
            .filter(|answer| answer.typed)
        else {
            push(target.key, "__typeFactsMissing", json!(true));
            continue;
        };
        push(
            target.key,
            "__expressionTypeFlags",
            flag_names(answer.constrained),
        );
        push(target.key, "__actualTypeFlags", flag_names(answer.actual));
        push(
            target.key,
            "__constrainedTypeDiffers",
            json!(answer.constrained_differs),
        );
        if let Some(contextual) = answer.contextual {
            push(target.key, "__contextualTypeFlags", flag_names(contextual));
        }
        push(
            target.key,
            "__possiblyUsedBeforeAssigned",
            json!(target.used_before_assigned || answer.declared_matches),
        );
    }
    for (index, key) in pending.defaults.iter().enumerate() {
        let same = reply
            .and_then(|reply| reply.defaults.get(index))
            .copied()
            .flatten();
        push(
            *key,
            "__lastTypeArgumentEqualsDefault",
            json!(same == Some(true)),
        );
    }
    for (index, key) in pending.deprecations.iter().enumerate() {
        let found = reply
            .and_then(|reply| reply.deprecations.get(index))
            .and_then(Option::as_ref);
        push(*key, "__deprecated", json!(found.is_some()));
        if let Some(found) = found {
            push(*key, "__deprecationReason", json!(found.reason.as_str()));
            if let Some(name) = &found.name {
                push(*key, "__deprecatedName", json!(name.as_str()));
            }
        }
    }
    for (index, key) in pending.returns.iter().enumerate() {
        match reply.and_then(|reply| reply.returns.get(index)) {
            Some(answer) => {
                push(*key, "__returnsContainClassType", json!(answer.class_type));
                push(*key, "__returnsContainThisType", json!(answer.this_type));
                push(*key, "__conciseBodyIsThisType", json!(answer.this_type));
            }
            None => push(*key, "__typeFactsMissing", json!(true)),
        }
    }
    for (index, (key, field)) in pending.assignable.iter().enumerate() {
        match reply
            .and_then(|reply| reply.assignable.get(index))
            .copied()
            .flatten()
        {
            Some(assignable) => push(*key, field, json!(assignable)),
            // An unknown getter/setter relation is never reported.
            None if *field == "__getterTypeAssignableToSetter" => push(*key, field, json!(true)),
            None => push(*key, "__typeFactsMissing", json!(true)),
        }
    }
    for node in nodes {
        stamp(node, &facts);
        // An array-likeness question the checker did not answer keeps the
        // rule silent.
        mark_untyped(node, &pending.array_like);
    }
}

fn mark_untyped(node: &mut LintNode, keys: &[Key]) {
    if !node.fields.contains_key("__type")
        && keys.iter().any(|&(kind, start, end)| {
            kind == node.kind && start == node.range.start && end == node.range.end
        })
    {
        node.fields.insert("__typeFactsMissing".into(), json!(true));
    }
    for child in node.children.values_mut() {
        mark_untyped(child, keys);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            mark_untyped(child, keys);
        }
    }
}

type Facts = FxHashMap<(u32, u32), Vec<(&'static str, &'static str, Value)>>;

fn stamp(node: &mut LintNode, facts: &Facts) {
    if let Some(list) = facts.get(&(node.range.start, node.range.end)) {
        for (kind, field, value) in list {
            if *kind == node.kind {
                node.fields.insert((*field).into(), value.clone());
            }
        }
    }
    for child in node.children.values_mut() {
        stamp(child, facts);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, facts);
        }
    }
}

fn flag_names(bits: u32) -> Value {
    let names = [
        (type_flags::ANY, "any"),
        (type_flags::UNKNOWN, "unknown"),
        (type_flags::NULL, "null"),
        (type_flags::UNDEFINED, "undefined"),
        (type_flags::VOID, "void"),
    ];
    Value::Array(
        names
            .iter()
            .filter(|(bit, _)| bits & bit != 0)
            .map(|(_, name)| json!(name))
            .collect(),
    )
}

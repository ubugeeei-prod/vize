//! TS-34 spec for `UndefinedRefs`: the unresolved-read rule and its naive
//! evaluator.
//!
//! ```text
//! read(e, n)        :- checked(e), n ∈ identifiers(e.content)        -- a fresh parse, no cache
//! visible(e, n)     :- n ∈ e.scope_vars
//! visible(e, n)     :- s ∈ ancestors*(e.scope), n ∈ bindings(s)      -- the FINAL scope tree
//! builtin(n)        :- js_global(n) ∨ vue_builtin(n) ∨ event_local(n) ∨ keyword(n)
//! undefined(n, off) :- read(e, n), ¬visible(e, n), ¬binding(n), ¬builtin(n),
//!                      off = e.base_offset + first_occurrence(e.content, n)
//! ```
//!
//! `binding(n)` reads the final `Bindings` table (a lower stratum), where the
//! drawer reads the binding map as it stood when the expression was walked;
//! `visible` walks the final scope tree from the recorded scope, where the
//! drawer asks its live cursor. Agreement therefore also proves that nothing
//! grows a binding or a scope after the template walk has judged a read.
//! The relation compares as a multiset of `(name, offset, context)` facts.

use std::collections::BTreeSet;

use vize_carton::CompactString;

use super::trace::CheckedExpression;
use crate::UndefinedRef;
use crate::builtins::{is_event_local, is_js_global, is_vue_builtin};
use crate::drawer::{extract_identifiers_oxc, is_keyword};
use crate::facts::bindings::Bindings;
use crate::facts::{BindingsTable, FactTable};
use crate::scope::{ScopeChain, ScopeId};

/// The context every template unresolved read reports.
pub const CONTEXT: &str = "template expression";

/// Whether `name` is visible from `scope` in the final scope tree: declared
/// in it or in any ancestor, over every parent edge.
fn visible_in_tree(scopes: &ScopeChain, scope: ScopeId, name: &str) -> bool {
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    let mut pending = vec![scope];
    while let Some(id) = pending.pop() {
        if !seen.insert(id.as_u32()) {
            continue;
        }
        let Some(node) = scopes.get_scope(id) else {
            continue;
        };
        if node.has_binding(name) {
            return true;
        }
        pending.extend(node.parents.iter().copied());
    }
    false
}

fn builtin(name: &str) -> bool {
    is_js_global(name) || is_vue_builtin(name) || is_event_local(name) || is_keyword(name)
}

/// The first occurrence of `name` in `content` that stands as a read — on
/// identifier boundaries and not after a `.` (whitespace skipped) — else its
/// first substring occurrence, else 0.
fn first_occurrence(content: &str, name: &str) -> u32 {
    let bytes = content.as_bytes();
    let continues = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$';
    let mut substring = None;
    for (at, _) in content.match_indices(name) {
        substring.get_or_insert(at);
        let before = at.checked_sub(1).map(|index| bytes[index]);
        let after = bytes.get(at + name.len()).copied();
        let member = bytes[..at]
            .iter()
            .rev()
            .find(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
            == Some(&b'.');
        if !before.is_some_and(continues) && !after.is_some_and(continues) && !member {
            return at as u32;
        }
    }
    substring.map_or(0, |at| at as u32)
}

/// Evaluate the rule over one artifact's checked expressions.
#[must_use]
pub fn evaluate(
    checked: &[CheckedExpression],
    scopes: &ScopeChain,
    bindings: &FactTable<Bindings>,
) -> Vec<UndefinedRef> {
    let mut facts = Vec::new();
    for expression in checked {
        for name in extract_identifiers_oxc(&expression.content) {
            let visible = expression.scope_vars.contains(&name)
                || visible_in_tree(scopes, expression.scope, &name);
            if visible || bindings.contains_binding(&name) || builtin(&name) {
                continue;
            }
            let offset = expression.base_offset + first_occurrence(&expression.content, &name);
            facts.push(UndefinedRef {
                name,
                offset,
                context: CompactString::new(CONTEXT),
            });
        }
    }
    facts
}

/// The multiset form both sides compare in: `(offset, name, context)`, sorted.
#[must_use]
pub fn canonical(
    facts: impl IntoIterator<Item = UndefinedRef>,
) -> Vec<(u32, CompactString, CompactString)> {
    let mut rows: Vec<_> = facts
        .into_iter()
        .map(|fact| (fact.offset, fact.name, fact.context))
        .collect();
    rows.sort();
    rows
}

#[cfg(test)]
mod tests {
    use super::first_occurrence;

    #[test]
    fn the_offset_rule_prefers_an_identifier_boundary() {
        assert_eq!(first_occurrence("items.length + item", "item"), 15);
        assert_eq!(first_occurrence("$item + xitem", "item"), 1);
        assert_eq!(first_occurrence("a + b", "zz"), 0);
        assert_eq!(first_occurrence("fooBar", "Bar"), 3);
        assert_eq!(first_occurrence("a. item + item", "item"), 10);
    }
}

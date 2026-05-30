//! Helper functions for Vue template analysis.
//!
//! Provides utilities for:
//! - Component and directive detection (`keywords`)
//! - Identifier extraction from expressions (`identifiers`)
//! - v-for expression parsing (`v_for`)
//! - v-slot and inline callback parameter extraction (`slots`)

mod identifiers;
mod keywords;
mod slots;
mod v_for;

pub use identifiers::{extract_identifiers_oxc, strip_js_comments};
pub use keywords::{is_builtin_directive, is_component_tag, is_keyword};
pub use slots::{extract_inline_callback_params, extract_slot_props};
pub use v_for::{VForScopeAliases, parse_v_for_expression, parse_v_for_scope_expression};

use vize_carton::{CompactString, String};

/// Which conditional directive an element carries within a `v-if` chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalKind {
    If,
    ElseIf,
    Else,
}

/// Build a TypeScript narrowing guard for one branch of a `v-if` chain.
///
/// `previous_conditions` are the conditions of the preceding `v-if` /
/// `v-else-if` branches; each is negated. `current_condition` is the branch's
/// own condition (`None` for `v-else`). The result mirrors the runtime control
/// flow so that discriminated-union narrowing reaches the branch body, e.g. a
/// `v-else` after `v-if="x.kind === 'a'"` yields `!(x.kind === 'a')`.
pub fn build_branch_guard(
    previous_conditions: &[CompactString],
    current_condition: Option<&str>,
) -> Option<CompactString> {
    if previous_conditions.is_empty() && current_condition.is_none() {
        return None;
    }

    let mut guard = String::default();
    let mut has_part = false;

    for previous in previous_conditions {
        if has_part {
            guard.push_str(" && ");
        }
        guard.push_str("!(");
        guard.push_str(previous.as_str());
        guard.push(')');
        has_part = true;
    }

    if let Some(condition) = current_condition {
        if has_part {
            guard.push_str(" && ");
        }
        guard.push('(');
        guard.push_str(condition);
        guard.push(')');
    }

    Some(CompactString::new(guard.as_str()))
}

/// Fast identifier validation using bytes
#[inline]
pub fn is_valid_identifier_fast(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_alphabetic() && first != b'_' && first != b'$' {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
}

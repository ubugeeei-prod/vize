//! Visibility of template-scope bindings at a template-relative offset.

use vize_croquis::{Croquis, Scope, ScopeKind};

/// Whether `name` is already bound where a template expression uses it.
///
/// `summary.bindings` answers for every script and setup binding and carries no
/// offset, so the offset-sensitive part of this question is only ever about the
/// scopes the *template* introduces: `v-for` aliases, `v-slot` props and the
/// `$event` an event handler binds.
///
/// `Croquis::bindings_visible_at` cannot answer it on its own. It starts from
/// `scope_at_offset`, which picks the smallest-span scope containing the offset
/// out of one flat table that mixes script scopes — whose spans are measured
/// over the script text — with template scopes, whose spans are measured over
/// the template text. The two ranges overlap for every template offset below
/// the script's length, the script scope usually wins on span size because it
/// stops at the end of the script, and `bindings_visible_at` then walks that
/// scope's parents, which never include the enclosing `v-for`.
///
/// A `<script setup>` of 140 characters and a `v-for` spanning template offsets
/// 13..209 is the measured case (#4423, elk `CommonRouteTabs.vue`): the alias
/// read at template offset 103 resolved against the script scope and reported
/// `TS2339`, while the same alias read at offset 148 — past the script's length
/// — resolved correctly. Inserting one HTML comment ahead of the element moved
/// every offset past 140 and the diagnostics disappeared, which is the shape of
/// the collision rather than of a real undeclared name.
///
/// So ask the template scopes directly. They nest properly among themselves, so
/// every one whose span contains the offset encloses that offset and there is
/// no smallest-span tie-break to get wrong. This only ever *adds* a reason for a
/// name to count as bound, so it cannot introduce a diagnostic the previous rule
/// did not already produce.
pub(super) fn is_visible_template_binding(
    summary: &Croquis,
    name: &str,
    template_offset: u32,
) -> bool {
    summary.bindings.contains(name)
        || binds_in_enclosing_template_scope(summary, name, template_offset)
}

pub(super) fn is_inside_template_scope(summary: &Croquis, template_offset: u32) -> bool {
    summary
        .scopes
        .iter()
        .any(|scope| is_active_template_scope(scope, template_offset))
}

fn binds_in_enclosing_template_scope(summary: &Croquis, name: &str, template_offset: u32) -> bool {
    summary.scopes.iter().any(|scope| {
        is_active_template_scope(scope, template_offset)
            && scope.bindings().any(|(binding, data)| {
                binding == name && data.declaration_offset <= template_offset
            })
    })
}

fn is_active_template_scope(scope: &Scope, template_offset: u32) -> bool {
    if !is_template_introduced_scope(scope.kind) || !scope.span.contains(template_offset) {
        return false;
    }

    let mut has_bindings = false;
    for (_, binding) in scope.bindings() {
        has_bindings = true;
        if binding.declaration_offset <= template_offset {
            return true;
        }
    }
    !has_bindings
}

/// Scope kinds a template — and only a template — introduces, so their spans are
/// always template-relative and safe to test against a template offset.
///
/// `Callback` is deliberately absent: `<script setup>` and template expressions
/// both create it, so its span may be measured over either text, and a script
/// callback whose range happens to cover a template offset would suppress a real
/// diagnostic.
fn is_template_introduced_scope(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
    )
}

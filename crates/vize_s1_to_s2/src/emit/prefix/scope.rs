//! The two scope notions the shipped lane consults while prefixing.
//!
//! - The **transform scope** is what `TransformContext::is_in_scope` saw
//!   when `process_expression` ran: each binding from a destructured
//!   `v-for` value, raw key/index aliases, and slot prop names from
//!   `extract_slot_prop_names`. It decides *whether* a name is prefixed.
//! - The **codegen slot params** are what `CodegenContext::is_slot_param`
//!   saw: `extract_destructure_params` over `v-for` aliases and slot
//!   params. They drive the codegen-time strips of prefixes the transform
//!   applied without knowing the pattern names, and the dynamic-argument
//!   special cases.
//!
//! Both are stacks that follow the emit walk exactly where the shipped
//! traversals entered and exited them.
//!
//! The default lane (no `prefix_identifiers`) only needs the codegen
//! slot-param *membership* for its dynamic-key spellings, so it records
//! the raw alias / slot patterns instead and asks
//! [`super::params::destructure_params_contain`] per query: no list is
//! materialised, and the allocation gate's window stays at its baseline.
//!
//! The binding table (P2-11 installment 86) decides *which* prefix a
//! free name gets: the shipped `get_identifier_prefix` in non-inline mode.

use alloc::vec::Vec as StdVec;

use vize_s0::{SmallVec, String};

use super::super::options::{BindingKind, BindingTable, ReactiveRead};
use super::globals::is_scope_chain_global;

#[derive(Default)]
pub(in crate::emit) struct PrefixScope<'b> {
    transform: StdVec<String>,
    slot_params: StdVec<String>,
    patterns: SmallVec<[String; 4]>,
    bindings: Option<&'b BindingTable>,
    prefix_identifiers: bool,
    is_ts: bool,
    inline: bool,
}

#[derive(Clone, Copy)]
pub(in crate::emit) struct ScopeMark {
    transform: usize,
    slot_params: usize,
    patterns: usize,
}

impl<'b> PrefixScope<'b> {
    pub(in crate::emit) fn new(
        bindings: Option<&'b BindingTable>,
        prefix_identifiers: bool,
        is_ts: bool,
        inline: bool,
    ) -> Self {
        Self {
            bindings,
            prefix_identifiers,
            is_ts,
            inline,
            ..Self::default()
        }
    }

    /// Whether the render function is inlined into `setup()`.
    pub(in crate::emit) fn inline(&self) -> bool {
        self.inline
    }

    /// The transform's `is_ref_binding`: an inline-mode ref is read
    /// through `.value`. The script's reactivity tracker answers first (a
    /// tracked `computed`/`ref`/`toRef` reads through `.value`, any other
    /// tracked kind does not), then the binding kind (P3-17).
    pub(super) fn is_ref_binding(&self, name: &str) -> bool {
        let Some(table) = self.bindings.filter(|_| self.inline) else {
            return false;
        };
        match table.reactive_read(name) {
            Some(read) => read == ReactiveRead::Value,
            None => table.kind(name) == Some(BindingKind::SetupRef),
        }
    }

    /// The codegen visitor's `is_ref`: the shipped codegen never sees the
    /// reactivity tracker and reads the binding kind alone.
    pub(super) fn codegen_is_ref_binding(&self, name: &str) -> bool {
        self.inline
            && self
                .bindings
                .and_then(|table| table.kind(name))
                .is_some_and(|kind| kind == BindingKind::SetupRef)
    }

    /// Whether `name` reads a binding that cannot change at runtime, as
    /// the codegen sees it: only an inline render function leaves the
    /// name bare enough for the lookup to hit.
    pub(in crate::emit) fn reads_constant_binding(&self, name: &str) -> bool {
        self.inline
            && self
                .bindings
                .and_then(|table| table.kind(name))
                .is_some_and(|kind| {
                    matches!(kind, BindingKind::LiteralConst | BindingKind::SetupConst)
                })
    }

    /// `should_ref_runtime_binding`: an inline `ref="name"` naming a
    /// setup binding the script can write is emitted as the
    /// `ref_key: "name", ref: name` pair, so the runtime's `setRef` can
    /// write back into `instance.refs` — what `useTemplateRef` reads.
    /// The same three kinds as [`PrefixScope::writes_through_value`],
    /// asked of a template ref rather than of an assignment target.
    pub(in crate::emit) fn writes_template_ref(&self, name: &str) -> bool {
        self.inline
            && self
                .bindings
                .and_then(|table| table.kind(name))
                .is_some_and(|kind| {
                    matches!(
                        kind,
                        BindingKind::SetupLet | BindingKind::SetupMaybeRef | BindingKind::SetupRef
                    )
                })
    }

    /// The codegen visitor's `needs_value`: an inline-mode assignment to
    /// a setup binding writes through `.value`.
    pub(super) fn writes_through_value(&self, name: &str) -> bool {
        self.inline
            && self
                .bindings
                .and_then(|table| table.kind(name))
                .is_some_and(|kind| {
                    matches!(
                        kind,
                        BindingKind::SetupLet | BindingKind::SetupMaybeRef | BindingKind::SetupRef
                    )
                })
    }

    /// The transform's `needs_unref`: a `let` or maybe-ref setup binding
    /// captured by an inline render closure is read through `_unref(…)` —
    /// unless the reactivity tracker knows its kind (P3-17).
    pub(super) fn needs_unref(&self, name: &str) -> bool {
        self.codegen_needs_unref(name)
            && self
                .bindings
                .is_some_and(|table| table.reactive_read(name).is_none())
    }

    /// The codegen visitor's `needs_unref`: binding kind alone.
    pub(super) fn codegen_needs_unref(&self, name: &str) -> bool {
        self.inline
            && self
                .bindings
                .and_then(|table| table.kind(name))
                .is_some_and(|kind| {
                    matches!(kind, BindingKind::SetupLet | BindingKind::SetupMaybeRef)
                })
    }

    /// Whether free identifiers are rewritten at all (`prefix_identifiers`).
    pub(in crate::emit) fn prefixes_identifiers(&self) -> bool {
        self.prefix_identifiers
    }

    /// Whether expressions are type-erased first (`is_ts`).
    pub(in crate::emit) fn is_ts(&self) -> bool {
        self.is_ts
    }

    /// The binding table the emit runs under, if any.
    pub(in crate::emit) fn bindings(&self) -> Option<&'b BindingTable> {
        self.bindings
    }

    pub(in crate::emit) fn mark(&self) -> ScopeMark {
        ScopeMark {
            transform: self.transform.len(),
            slot_params: self.slot_params.len(),
            patterns: self.patterns.len(),
        }
    }

    pub(in crate::emit) fn pop(&mut self, mark: ScopeMark) {
        self.transform.truncate(mark.transform);
        self.slot_params.truncate(mark.slot_params);
        self.patterns.truncate(mark.patterns);
    }

    pub(super) fn push_event(&mut self) {
        self.transform.push(String::from("$event"));
    }

    /// The default lane's record of a `v-for` alias or slot pattern.
    pub(in crate::emit) fn push_pattern(&mut self, pattern: &str) {
        let trimmed = pattern.trim();
        if !trimmed.is_empty() {
            self.patterns.push(String::from(trimmed));
        }
    }

    /// `CodegenContext::is_slot_param` over the recorded patterns.
    pub(in crate::emit) fn binds_in_pattern(&self, name: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| super::params::destructure_params_contain(pattern.as_str(), name))
    }

    /// `TransformContext::enter_v_for_scope` + the codegen callback params.
    pub(in crate::emit) fn push_for(&mut self, aliases: [Option<&str>; 3]) {
        for (index, alias) in aliases.into_iter().enumerate() {
            let Some(alias) = alias else { continue };
            if alias.is_empty() {
                continue;
            }
            // The shipped scope chain registers each binding of a
            // destructured value pattern before process_expression runs.
            // Slot text only strips `_ctx.` at codegen, so a prop with the
            // same name must be shadowed here rather than prefixed first.
            if index == 0 && alias.trim_start().starts_with(['{', '[']) {
                self.transform
                    .extend(super::params::extract_slot_prop_names(alias));
            } else {
                self.transform.push(String::from(alias));
            }
            super::params::extract_destructure_params(alias.trim(), &mut self.slot_params);
        }
    }

    /// `enter_v_slot_scope_if_needed` + the codegen slot params.
    pub(in crate::emit) fn push_slot(&mut self, params: &str) {
        self.transform
            .extend(super::params::extract_slot_prop_names(params));
        super::params::extract_destructure_params(params.trim(), &mut self.slot_params);
    }

    pub(in crate::emit) fn is_in_transform_scope(&self, name: &str) -> bool {
        self.transform.iter().any(|entry| entry.as_str() == name)
    }

    pub(in crate::emit) fn is_slot_param(&self, name: &str) -> bool {
        self.slot_params.iter().any(|entry| entry.as_str() == name)
    }

    pub(in crate::emit) fn has_slot_params(&self) -> bool {
        !self.slot_params.is_empty()
    }

    pub(in crate::emit) fn has_codegen_scope_params(&self) -> bool {
        !self.slot_params.is_empty() || !self.patterns.is_empty()
    }

    pub(in crate::emit) fn slot_params(&self) -> &[String] {
        &self.slot_params
    }

    /// `get_identifier_prefix` in non-inline mode: `None` for the seeded
    /// JS globals and transform-scope names, the binding's render-function prefix
    /// (`$props.` for props) when the table names it, `_ctx.` otherwise.
    pub(super) fn identifier_prefix(&self, name: &str) -> Option<&'static str> {
        // The shipped lane only reaches `get_identifier_prefix` under
        // `prefix_identifiers`; a TS-only lane erases types and prefixes
        // nothing.
        if !self.prefix_identifiers {
            return None;
        }
        if is_scope_chain_global(name) || self.is_in_transform_scope(name) {
            return None;
        }
        self.transform_prefix(name)
    }

    /// `get_identifier_prefix`'s binding arm: inline mode reads props off
    /// the setup-local `__props` and every other setup binding straight
    /// from the closure — `None`, which is what lets the collector fall
    /// through to the `.value` / `_unref` decisions.
    fn transform_prefix(&self, name: &str) -> Option<&'static str> {
        match self.bindings.and_then(|table| table.kind(name)) {
            Some(kind) if kind.is_props() => Some(if self.inline { "__props." } else { "$props." }),
            Some(kind) => (!self.inline).then(|| kind.non_inline_template_prefix()),
            None => Some("_ctx."),
        }
    }

    /// The codegen `IdentifierVisitor` prefix (no scope or allowlist check;
    /// the visitor applies those first): the binding's non-inline prefix,
    /// `_ctx.` for names the table does not know.
    pub(super) fn codegen_prefix(&self, name: &str) -> &'static str {
        match self.bindings.and_then(|table| table.kind(name)) {
            // Inline mode reads props off the render closure's own
            // `$props` and every other setup binding directly.
            Some(kind) if self.inline => {
                if kind.is_props() {
                    "$props."
                } else {
                    ""
                }
            }
            Some(kind) if kind.is_props() => "$props.",
            Some(kind) => kind.non_inline_template_prefix(),
            None => "_ctx.",
        }
    }
}

#[cfg(test)]
mod tests;

//! The two scope notions the shipped lane consults while prefixing.
//!
//! - The **transform scope** is what `TransformContext::is_in_scope` saw
//!   when `process_expression` ran: `v-for` aliases as whole strings (a
//!   destructuring pattern never matches an identifier) and slot prop
//!   names from `extract_slot_prop_names`. It decides *whether* a name is
//!   prefixed.
//! - The **codegen slot params** are what `CodegenContext::is_slot_param`
//!   saw: `extract_destructure_params` over `v-for` aliases and slot
//!   params. They drive the codegen-time strips of prefixes the transform
//!   applied without knowing the pattern names, and the dynamic-argument
//!   special cases.
//!
//! Both are stacks that follow the emit walk exactly where the shipped
//! traversals entered and exited them.

use alloc::vec::Vec as StdVec;

use vize_s0::String;

#[derive(Default)]
pub(in crate::emit) struct PrefixScope {
    transform: StdVec<String>,
    slot_params: StdVec<String>,
}

#[derive(Clone, Copy)]
pub(in crate::emit) struct ScopeMark {
    transform: usize,
    slot_params: usize,
}

impl PrefixScope {
    pub(in crate::emit) fn mark(&self) -> ScopeMark {
        ScopeMark {
            transform: self.transform.len(),
            slot_params: self.slot_params.len(),
        }
    }

    pub(in crate::emit) fn pop(&mut self, mark: ScopeMark) {
        self.transform.truncate(mark.transform);
        self.slot_params.truncate(mark.slot_params);
    }

    /// `TransformContext::enter_v_for_scope` + the codegen callback params.
    pub(in crate::emit) fn push_for(&mut self, aliases: [Option<&str>; 3]) {
        for alias in aliases.into_iter().flatten() {
            if alias.is_empty() {
                continue;
            }
            self.transform.push(String::from(alias));
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

    pub(in crate::emit) fn slot_params(&self) -> &[String] {
        &self.slot_params
    }

    /// `get_identifier_prefix` without binding metadata: `None` for
    /// globals and transform-scope names, `_ctx.` otherwise.
    pub(super) fn identifier_prefix(&self, name: &str) -> Option<&'static str> {
        if super::globals::is_global_allowed(name) || self.is_in_transform_scope(name) {
            return None;
        }
        Some("_ctx.")
    }
}

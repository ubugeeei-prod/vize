//! Shared emitter context helpers kept out of `emit.rs` for the source budget.

use vize_davinci::id::NodeId;

use super::{EmitCx, EmitError};

impl EmitCx<'_> {
    pub(super) fn scope_mark(&self) -> usize {
        self.scope_names.len()
    }

    pub(super) fn push_scope(&mut self, id: Option<NodeId>) -> usize {
        let mark = self.scope_mark();
        if let Some(facts) = id.and_then(|id| self.scopes.get(id)) {
            for binding in facts.bindings.iter() {
                self.scope_names.push(binding.name.clone());
            }
        }
        mark
    }

    pub(super) fn pop_scope(&mut self, mark: usize) {
        self.scope_names.truncate(mark);
    }

    pub(super) fn is_scope_name(&self, source: &str) -> bool {
        self.scope_names.iter().any(|name| name.as_str() == source)
    }

    pub(super) fn with_static_vnode_hoist<T>(
        &mut self,
        enabled: bool,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.hoist_static_vnodes;
        self.hoist_static_vnodes = previous || enabled;
        let result = write(self);
        self.hoist_static_vnodes = previous;
        result
    }

    pub(super) fn with_static_vnode_hoist_exact<T>(
        &mut self,
        enabled: bool,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.hoist_static_vnodes;
        self.hoist_static_vnodes = enabled;
        let result = write(self);
        self.hoist_static_vnodes = previous;
        result
    }

    pub(super) fn with_once_element<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.once_element_depth;
        if self.once_depth > 0 {
            self.once_element_depth = self.once_element_depth.saturating_add(1);
        }
        let result = write(self);
        self.once_element_depth = previous;
        result
    }
}

//! The synthetic binding a `<component :is="expr">` resolves through when
//! `expr` is not a plain component identifier.

use vize_carton::String;

use super::super::super::Drawer;
use crate::scope::ScopeKind;

impl Drawer {
    /// Whether the current template position is outside every scope that
    /// introduces bindings an alias declared at the template root could not
    /// see: loops, slots, pattern arms and handler or callback bodies.
    pub(super) fn in_template_root_scope(&self) -> bool {
        let mut current = Some(self.croquis.scopes.current_id());
        while let Some(id) = current {
            let Some(scope) = self.croquis.scopes.get_scope(id) else {
                return false;
            };
            if matches!(
                scope.kind,
                ScopeKind::VFor
                    | ScopeKind::VSlot
                    | ScopeKind::VMatch
                    | ScopeKind::VWhen
                    | ScopeKind::EventHandler
                    | ScopeKind::Callback
                    | ScopeKind::Closure
            ) {
                return false;
            }
            current = scope.parent();
        }
        true
    }
}

/// The generated binding a dynamic `<component :is="expr">` resolves through
/// when `expr` is not a plain component identifier.
pub fn dynamic_component_alias(element_start: u32) -> String {
    vize_carton::cstr!("__vize_dynamic_is_{element_start}")
}

/// Whether a component usage name is a [`dynamic_component_alias`].
pub fn is_dynamic_component_alias(name: &str) -> bool {
    name.starts_with("__vize_dynamic_is_")
}

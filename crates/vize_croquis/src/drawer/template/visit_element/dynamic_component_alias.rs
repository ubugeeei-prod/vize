//! The synthetic binding a `<component :is="expr">` resolves through when
//! `expr` is not a plain component identifier.

use vize_carton::String;

use super::super::super::Drawer;
use crate::scope::ScopeKind;

impl Drawer {
    /// A simple computed target may be captured at the template root only
    /// when both retained-AST identifiers have the same actual script owner
    /// at this usage. Slot/loop shadows and type-only imports are not values.
    pub(super) fn dynamic_index_has_only_script_values(
        &self,
        expression: &vize_relief::ExpressionNode<'_>,
    ) -> bool {
        use oxc_ast::ast::Expression;
        let vize_relief::ExpressionNode::Simple(expression) = expression else {
            return false;
        };
        let Some(js) = expression.js_ast.and_then(|js| js.as_expression()) else {
            return false;
        };
        let Expression::ComputedMemberExpression(member) = js.ast else {
            return false;
        };
        let (Expression::Identifier(object), Expression::Identifier(key)) =
            (&member.object, &member.expression)
        else {
            return false;
        };
        js.raw == expression.content
            && self.croquis.bindings.is_script_setup
            && [object.name.as_str(), key.name.as_str()]
                .iter()
                .all(|name| {
                    self.croquis.bindings.contains(name)
                        && self.croquis.scopes.lookup(name).is_none_or(|(scope, _)| {
                            matches!(
                                scope.kind,
                                ScopeKind::Module
                                    | ScopeKind::ScriptSetup
                                    | ScopeKind::ExternalModule
                            )
                        })
                })
    }

    /// New admissions must not shadow an authored value or be captured by
    /// an already active template binding with the generated alias spelling.
    pub(super) fn new_dynamic_alias_is_unbound(&self, start: u32) -> bool {
        let alias = dynamic_component_alias(start);
        !self.croquis.bindings.contains(alias.as_str())
            && self.croquis.scopes.lookup(alias.as_str()).is_none()
    }

    // Ordinary setup declarations live in BindingMetadata; ScopeChain only
    // adds imports and lexical/template bindings. A present nearest binding
    // must still belong to a script scope; absence never admits a name that
    // the actual setup value table does not own.
    pub(super) fn dynamic_target_has_script_root(&self, target: &str) -> bool {
        let Some(root) = target.split('.').next() else {
            return false;
        };
        self.croquis.bindings.is_script_setup
            && self.croquis.bindings.contains(root)
            && self.croquis.scopes.lookup(root).is_none_or(|(scope, _)| {
                matches!(
                    scope.kind,
                    ScopeKind::Module | ScopeKind::ScriptSetup | ScopeKind::ExternalModule
                )
            })
    }

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

#[cfg(test)]
mod tests;

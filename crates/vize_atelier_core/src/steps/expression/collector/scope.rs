//! Existing lexical scope storage for the expression collector.

use vize_l0::{FxHashSet, String};
use vize_relief::ExpressionScope;

use super::IdentifierCollector;

impl<'ast> ExpressionScope<'ast> for IdentifierCollector<'_, '_> {
    fn push_scope(&mut self) {
        self.local_scopes.push(FxHashSet::default());
    }
    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }
    fn add_local(&mut self, name: &str) {
        // The walker pushes a scope before any binding; without one there is
        // nowhere to record the name.
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(String::new(name));
        }
    }
}

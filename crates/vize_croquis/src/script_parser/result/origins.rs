//! Snapshot provenance follows lexical declarations, including closure captures.

use super::{CompactString, ReactiveValueOrigin, ScopeId, ScriptParseResult};

impl ScriptParseResult {
    pub(crate) fn record_reactive_origin(
        &mut self,
        name: CompactString,
        origin: ReactiveValueOrigin,
    ) {
        self.scoped_reactive_value_origins
            .entry(self.scopes.current_id())
            .or_default()
            .insert(name.clone(), origin.clone());
        self.reactive_value_origins.insert(name, origin);
    }

    fn reactive_origin_scope(&self, name: &str) -> Option<ScopeId> {
        let mut current = Some(self.scopes.current_id());
        while let Some(id) = current {
            let scope = self.scopes.get_scope(id)?;
            if self
                .scoped_reactive_value_origins
                .get(&id)
                .is_some_and(|origins| origins.contains_key(name))
            {
                return Some(id);
            }
            // A local, parameter or catch binding hides any outer snapshot.
            if scope.get_binding(name).is_some() {
                return None;
            }
            current = scope.parent();
        }
        None
    }

    pub(crate) fn reactive_origin(&self, name: &str) -> Option<&ReactiveValueOrigin> {
        self.scoped_reactive_value_origins
            .get(&self.reactive_origin_scope(name)?)?
            .get(name)
    }

    pub(crate) fn clear_reactive_origin(&mut self, name: &str) {
        let Some(id) = self.reactive_origin_scope(name) else {
            return;
        };
        if let Some(origins) = self.scoped_reactive_value_origins.get_mut(&id) {
            origins.remove(name);
        }
        self.reactive_value_origins.remove(name);
    }
}

//! Ref initializer facts and preserved snapshot identity follow lexical scopes.

use super::{CompactString, ReactiveValueOrigin, RefValueSourceKind, ScriptParseResult};

impl ScriptParseResult {
    pub(crate) fn record_ref_value_source(&mut self, name: &str, kind: RefValueSourceKind) {
        self.ref_value_sources
            .entry(self.scopes.current_id())
            .or_default()
            .insert(CompactString::new(name), kind);
    }

    pub(crate) fn ref_value_source_kind(&self, name: &str) -> RefValueSourceKind {
        if self.ref_value_sources.is_empty() {
            return RefValueSourceKind::Other;
        }
        let mut current = Some(self.scopes.current_id());
        while let Some(id) = current {
            let Some(scope) = self.scopes.get_scope(id) else {
                break;
            };
            if let Some(kind) = self
                .ref_value_sources
                .get(&id)
                .and_then(|sources| sources.get(name))
            {
                return *kind;
            }
            if scope.get_binding(name).is_some() {
                break;
            }
            current = scope.parent();
        }
        RefValueSourceKind::Other
    }

    pub(super) fn origin_keeps_live_ref_value(&self, origin: &ReactiveValueOrigin) -> bool {
        let source = match origin {
            ReactiveValueOrigin::RefValue { source_name }
            | ReactiveValueOrigin::ReactiveProperty { source_name, .. }
            | ReactiveValueOrigin::PlainAlias { source_name } => source_name,
            _ => return false,
        };
        let root = source
            .split_once(".value")
            .map_or(source.as_str(), |(root, _)| root);
        self.ref_value_source_kind(root) == RefValueSourceKind::Live
    }

    pub(crate) fn record_live_ref_origin(&mut self, name: &str, live: bool) {
        let scope = self.scopes.current_id();
        if live {
            self.live_ref_value_origins
                .entry(scope)
                .or_default()
                .insert(CompactString::new(name));
        } else if let Some(origins) = self.live_ref_value_origins.get_mut(&scope) {
            origins.remove(name);
        }
    }

    pub(crate) fn keeps_live_ref_origin(&self, name: &str) -> bool {
        self.reactive_origin_scope(name)
            .and_then(|scope| self.live_ref_value_origins.get(&scope))
            .is_some_and(|origins| origins.contains(name))
    }
}

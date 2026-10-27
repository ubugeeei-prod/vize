//! Indexed, iterative identity finalization. TDZ belongs to each authored edge/use.

use super::{
    Declaration, Origin, ResolvedValueBinding, ValueBindingKey, ValueBindingOrigins, ValueReference,
};
use crate::scope::ScopeId;
use oxc_span::Span;
use vize_carton::{CompactString, FxHashSet};

impl ValueBindingOrigins {
    pub(crate) fn declaration_key(&self, scope: ScopeId, span: Span) -> Option<ValueBindingKey> {
        let key = ValueBindingKey {
            scope,
            start: span.start,
            end: span.end,
        };
        self.key_names.contains_key(&key).then_some(key)
    }

    pub(super) fn lookup(&self, reference: &ValueReference) -> Option<&Declaration> {
        if reference.end < reference.start {
            return None;
        }
        let mut scope = Some(reference.scope);
        while let Some(id) = scope {
            if let Some(declaration) = self
                .declarations
                .get(&id)
                .and_then(|table| table.get(reference.name.as_str()))
            {
                return Some(declaration);
            }
            scope = self.parents.get(&id).copied().flatten();
        }
        None
    }

    fn by_key(&self, key: ValueBindingKey) -> Option<&Declaration> {
        let name = self.key_names.get(&key)?;
        self.declarations.get(&key.scope)?.get(name)
    }

    /// Canonical-only joins are value-authority joins. A None canonical keeps
    /// declaration identity for diagnostics, never permission to reuse a shape.
    pub(crate) fn resolve(&self, reference: &ValueReference) -> Option<ResolvedValueBinding> {
        let declaration = self.lookup(reference)?;
        let canonical = if self.is_complete() && declaration.initialized <= reference.start {
            self.canonical_keys.get(&declaration.key).copied().flatten()
        } else {
            None
        };
        let import = canonical
            .and_then(|key| self.by_key(key))
            .and_then(|declaration| match &declaration.origin {
                Origin::Import(import) if import.value => Some(import.clone()),
                _ => None,
            });
        Some(ResolvedValueBinding {
            binding: declaration.key,
            canonical,
            import,
        })
    }

    pub(super) fn finalize_identities(&mut self) {
        let keys: Vec<_> = self.key_names.keys().copied().collect();
        for key in keys {
            self.finalize_key(key);
        }
        for key in &self.member_mutated {
            if let Some(canonical) = self.canonical_keys.get(key).copied().flatten() {
                self.canonical_member_writes.insert(canonical);
            }
        }
    }

    fn finalize_key(&mut self, start: ValueBindingKey) {
        if self.canonical_keys.contains_key(&start) {
            return;
        }
        let mut current = start;
        let mut path = Vec::new();
        let mut seen = FxHashSet::default();
        let result = loop {
            if let Some(cached) = self.canonical_keys.get(&current) {
                break *cached;
            }
            if !seen.insert(current) {
                break None;
            }
            path.push(current);
            let Some(declaration) = self.by_key(current) else {
                break None;
            };
            if declaration.ambiguous || !declaration.immutable || self.mutated.contains(&current) {
                break None;
            }
            match &declaration.origin {
                Origin::Alias(reference) => {
                    let Some(target) = self.lookup(reference) else {
                        break None;
                    };
                    // This edge is evaluated at the alias initializer's original
                    // offset. A later use cannot repair an earlier TDZ edge.
                    if target.initialized > reference.start {
                        break None;
                    }
                    current = target.key;
                }
                Origin::Import(import) if !import.value => break None,
                _ => break Some(current),
            }
        };
        for key in path {
            self.canonical_keys.insert(key, result);
        }
    }

    pub(crate) fn by_key_is_stable(&self, key: ValueBindingKey) -> bool {
        self.is_complete() && self.canonical_keys.get(&key).is_some_and(Option::is_some)
    }

    pub(crate) fn has_member_write(&self, canonical: ValueBindingKey) -> bool {
        self.canonical_member_writes.contains(&canonical)
    }

    pub(crate) fn component_binding_name(&self, tag: &str) -> Option<Option<&str>> {
        let root = self.declarations.get(&self.root?)?;
        if let Some((name, _)) = root.get_key_value(tag) {
            return Some(Some(name.as_str()));
        }
        let normalized = CompactString::new(crate::naming::to_pascal_case(tag));
        self.component_names
            .get(&normalized)
            .map(|name| name.as_deref())
    }

    pub(crate) fn root_reference(&self, name: &str) -> Option<ValueReference> {
        Some(ValueReference::new(
            self.root?,
            name,
            Span::new(u32::MAX, u32::MAX),
        ))
    }
}

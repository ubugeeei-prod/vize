//! Private, original-Program value identities collected by the existing script walk.
//!
//! These records are not module-resolution evidence and are never serialized.
//! Keys belong to one store; merging two actual Programs refuses provenance.

use crate::scope::{ScopeChain, ScopeId, ScopeKind};
mod observe;
pub(crate) use observe::note_expression_mutation;

use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclaration, ImportDeclarationSpecifier,
    VariableDeclarationKind,
};
use oxc_span::{GetSpan, Span};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ValueBindingKey {
    pub scope: ScopeId,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct ValueReference {
    pub scope: ScopeId,
    pub name: CompactString,
    pub start: u32,
    pub end: u32,
}

impl ValueReference {
    pub(crate) fn new(scope: ScopeId, name: &str, span: Span) -> Self {
        Self {
            scope,
            name: CompactString::new(name),
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueImportExport {
    Named(CompactString),
    Namespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueImportOrigin {
    pub source: CompactString,
    pub export: ValueImportExport,
    pub value: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedValueBinding {
    pub binding: ValueBindingKey,
    pub canonical: Option<ValueBindingKey>,
    pub import: Option<ValueImportOrigin>,
}

#[derive(Debug, Clone)]
enum Origin {
    Opaque,
    Import(ValueImportOrigin),
    Alias(ValueReference),
}

#[derive(Debug, Clone)]
struct Declaration {
    key: ValueBindingKey,
    origin: Origin,
    initialized: u32,
    immutable: bool,
    ambiguous: bool,
}

/// One original parser domain. No AST, semantic node store, or source text is retained.
#[derive(Debug, Default)]
pub struct ValueBindingOrigins {
    root: Option<ScopeId>,
    parents: FxHashMap<ScopeId, Option<ScopeId>>,
    declarations: FxHashMap<ScopeId, FxHashMap<CompactString, Declaration>>,
    writes: Vec<(ValueReference, bool)>,
    mutated: FxHashSet<ValueBindingKey>,
    member_mutated: FxHashSet<ValueBindingKey>,
    refused: bool,
    admitted: bool,
    pub(crate) async_components: super::async_components::AsyncComponentBindings,
}

impl ValueBindingOrigins {
    pub(crate) fn new(root: ScopeId) -> Self {
        Self {
            root: Some(root),
            ..Self::default()
        }
    }

    pub(crate) fn refuse(&mut self) {
        self.refused = true;
    }

    pub(crate) fn admit(&mut self) {
        self.admitted = true;
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.root.is_some() && self.admitted && !self.refused
    }

    pub(crate) fn note_opaque_binding(&mut self, scope: ScopeId, name: &str, span: Span) {
        self.insert(scope, name, span, Origin::Opaque, 0, true);
    }

    fn insert(
        &mut self,
        scope: ScopeId,
        name: &str,
        span: Span,
        origin: Origin,
        initialized: u32,
        immutable: bool,
    ) {
        let key = ValueBindingKey {
            scope,
            start: span.start,
            end: span.end,
        };
        let table = self.declarations.entry(scope).or_default();
        if let Some(previous) = table.get_mut(name) {
            if previous.key != key {
                previous.ambiguous = true;
            }
            return;
        }
        table.insert(
            CompactString::new(name),
            Declaration {
                key,
                origin,
                initialized,
                immutable,
                ambiguous: false,
            },
        );
    }

    pub(crate) fn note_import(&mut self, scope: ScopeId, import: &ImportDeclaration<'_>) {
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            let (local, export, value) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(spec) => (
                    &spec.local,
                    ValueImportExport::Named(CompactString::new(spec.imported.name().as_str())),
                    !import.import_kind.is_type() && !spec.import_kind.is_type(),
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => (
                    &spec.local,
                    ValueImportExport::Named(CompactString::new("default")),
                    !import.import_kind.is_type(),
                ),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => (
                    &spec.local,
                    ValueImportExport::Namespace,
                    !import.import_kind.is_type(),
                ),
            };
            // Type-only declarations are retained as barriers, never value authority.
            self.insert(
                scope,
                local.name.as_str(),
                local.span,
                Origin::Import(ValueImportOrigin {
                    source: CompactString::new(import.source.value.as_str()),
                    export,
                    value,
                }),
                0,
                true,
            );
        }
    }

    pub(crate) fn note_declaration(
        &mut self,
        scope: ScopeId,
        pattern: &BindingPattern<'_>,
        kind: VariableDeclarationKind,
        init: Option<&Expression<'_>>,
    ) {
        if kind.is_var() {
            // The existing diagnostic scopes do not encode var hoisting.
            self.refuse();
        }
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                let origin = if kind.is_const() {
                    match init.map(Expression::get_inner_expression) {
                        Some(Expression::Identifier(reference)) => Origin::Alias(
                            ValueReference::new(scope, reference.name.as_str(), reference.span),
                        ),
                        _ => Origin::Opaque,
                    }
                } else {
                    Origin::Opaque
                };
                let initialized = init.map_or(id.span.end, |expression| expression.span().end);
                self.insert(
                    scope,
                    id.name.as_str(),
                    id.span,
                    origin,
                    initialized,
                    kind.is_const(),
                );
                if kind.is_const()
                    && let Some(init) = init
                {
                    self.async_components.note_initializer(
                        ValueBindingKey {
                            scope,
                            start: id.span.start,
                            end: id.span.end,
                        },
                        scope,
                        init,
                    );
                }
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.note_pattern_barriers(
                        scope,
                        &property.value,
                        kind.is_const(),
                        init.map_or(pattern.span().end, |expression| expression.span().end),
                    );
                }
                if let Some(rest) = &object.rest {
                    self.note_pattern_barriers(
                        scope,
                        &rest.argument,
                        kind.is_const(),
                        init.map_or(pattern.span().end, |expression| expression.span().end),
                    );
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.note_pattern_barriers(
                        scope,
                        element,
                        kind.is_const(),
                        init.map_or(pattern.span().end, |expression| expression.span().end),
                    );
                }
                if let Some(rest) = &array.rest {
                    self.note_pattern_barriers(
                        scope,
                        &rest.argument,
                        kind.is_const(),
                        init.map_or(pattern.span().end, |expression| expression.span().end),
                    );
                }
            }
            BindingPattern::AssignmentPattern(assignment) => self.note_pattern_barriers(
                scope,
                &assignment.left,
                kind.is_const(),
                init.map_or(pattern.span().end, |expression| expression.span().end),
            ),
        }
    }

    fn note_pattern_barriers(
        &mut self,
        scope: ScopeId,
        pattern: &BindingPattern<'_>,
        immutable: bool,
        initialized: u32,
    ) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self.insert(
                scope,
                id.name.as_str(),
                id.span,
                Origin::Opaque,
                initialized,
                immutable,
            ),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.note_pattern_barriers(scope, &property.value, immutable, initialized);
                }
                if let Some(rest) = &object.rest {
                    self.note_pattern_barriers(scope, &rest.argument, immutable, initialized);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.note_pattern_barriers(scope, element, immutable, initialized);
                }
                if let Some(rest) = &array.rest {
                    self.note_pattern_barriers(scope, &rest.argument, immutable, initialized);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.note_pattern_barriers(scope, &assignment.left, immutable, initialized)
            }
        }
    }

    pub(crate) fn note_write(&mut self, reference: ValueReference, member: bool) {
        self.writes.push((reference, member));
    }

    /// Finalize over existing scope metadata, not a second AST traversal.
    pub(crate) fn finish(&mut self, scopes: &ScopeChain) {
        let Some(root) = self.root else {
            return;
        };
        for scope in scopes.iter().filter(|scope| {
            scope.id.as_u32() >= root.as_u32() && scope.kind != ScopeKind::ExternalModule
        }) {
            self.parents.insert(scope.id, scope.parent());
            for (name, binding) in scope.bindings() {
                let table = self.declarations.entry(scope.id).or_default();
                table
                    .entry(CompactString::new(name))
                    .or_insert_with(|| Declaration {
                        key: ValueBindingKey {
                            scope: scope.id,
                            start: binding.declaration_offset,
                            end: binding.declaration_offset,
                        },
                        origin: Origin::Opaque,
                        initialized: 0,
                        immutable: false,
                        ambiguous: false,
                    });
            }
        }
        for (reference, member) in &self.writes {
            if let Some(key) = self.lookup(reference).map(|declaration| declaration.key) {
                if *member {
                    self.member_mutated.insert(key);
                } else {
                    self.mutated.insert(key);
                }
            }
        }
    }

    pub(crate) fn declaration_key(&self, scope: ScopeId, span: Span) -> Option<ValueBindingKey> {
        self.declarations
            .get(&scope)?
            .values()
            .find(|declaration| {
                declaration.key.start == span.start && declaration.key.end == span.end
            })
            .map(|declaration| declaration.key)
    }

    fn lookup(&self, reference: &ValueReference) -> Option<&Declaration> {
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

    pub(crate) fn resolve(&self, reference: &ValueReference) -> Option<ResolvedValueBinding> {
        let declaration = self.lookup(reference)?;
        let canonical = if self.is_complete() {
            self.canonical(declaration, reference.start, &mut FxHashSet::default())
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

    fn canonical(
        &self,
        declaration: &Declaration,
        use_start: u32,
        seen: &mut FxHashSet<ValueBindingKey>,
    ) -> Option<ValueBindingKey> {
        if !seen.insert(declaration.key)
            || declaration.ambiguous
            || !declaration.immutable
            || self.mutated.contains(&declaration.key)
            || declaration.initialized > use_start
        {
            return None;
        }
        match &declaration.origin {
            Origin::Alias(reference) => {
                self.canonical(self.lookup(reference)?, reference.start, seen)
            }
            Origin::Import(import) if !import.value => None,
            _ => Some(declaration.key),
        }
    }

    #[allow(
        dead_code,
        reason = "private identity API consumed by return-origin analysis"
    )]
    pub(crate) fn by_key_is_stable(&self, key: ValueBindingKey) -> bool {
        self.by_key(key).is_some_and(|declaration| {
            self.canonical(declaration, u32::MAX, &mut FxHashSet::default())
                .is_some()
        })
    }

    fn by_key(&self, key: ValueBindingKey) -> Option<&Declaration> {
        self.declarations
            .get(&key.scope)?
            .values()
            .find(|declaration| declaration.key == key)
    }

    pub(crate) fn has_member_write(&self, canonical: ValueBindingKey) -> bool {
        self.member_mutated.iter().any(|key| {
            self.by_key(*key).and_then(|declaration| {
                self.canonical(declaration, u32::MAX, &mut FxHashSet::default())
            }) == Some(canonical)
        })
    }

    pub(crate) fn binding_names(&self) -> impl Iterator<Item = &str> {
        self.root
            .and_then(|root| self.declarations.get(&root))
            .into_iter()
            .flat_map(|table| table.keys().map(CompactString::as_str))
    }

    pub(crate) fn root_reference(&self, name: &str) -> Option<ValueReference> {
        Some(ValueReference::new(
            self.root?,
            name,
            Span::new(u32::MAX, u32::MAX),
        ))
    }
}

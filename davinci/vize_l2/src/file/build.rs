//! Crate-private construction capability fed only by real language events.

use super::records::ProgramOrigin;
use super::{
    Declaration, DeclarationKind, Export, FileArtifact, FileIssue, FileIssueKind, Import,
    InitializerKind, Namespace, Reference, ReferenceTarget, RejectedFile, Scope, ScopeId,
    ScriptProfile, ScriptUnit, ScriptUnitId, TemplateIssue,
};
use crate::artifact::{ArtifactError, Builder};
use crate::resolution::{BindingId, ResolutionTable};
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, Span, String, id::NodeId};

pub(crate) struct Facts<'a> {
    pub units: Vec<ScriptUnit>,
    pub scopes: Vec<Scope>,
    pub declarations: Vec<Declaration>,
    pub references: Vec<Reference>,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
    pub issues: Vec<FileIssue>,
    pub expressions: SideTable<ScopedResolution<'a>>,
    pub template_issues: Vec<TemplateIssue>,
    pub template_walk: super::template::TemplateWalk,
    pub setup_annotations: Option<Box<Vec<crate::lang::js::file::setup::SetupAnnotationRecord>>>,
    names: Vec<Names>,
}

pub(crate) struct ScopedResolution<'a> {
    pub scope: ScopeId,
    pub table: ResolutionTable<'a>,
}

#[derive(Default)]
struct Names {
    value_names: BTreeMap<String, BindingId>,
    type_names: BTreeMap<String, BindingId>,
}

/// The neutral owner builder exposes only actual checked tree factories.
pub struct FileBuilder<'a> {
    pub(crate) source: &'a str,
    pub(crate) canonical: Builder<'a>,
    pub(crate) facts: Facts<'a>,
}

impl<'a> FileBuilder<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Result<Self, ArtifactError> {
        let canonical = Builder::new(allocator, source)?;
        Ok(Self {
            source,
            canonical,
            facts: Facts::new(source),
        })
    }

    pub fn text(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.canonical.text(content, span)
    }

    pub fn comment(&mut self, content: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.canonical.comment(content, span)
    }

    /// Transfer factory-checked fields; no second tree/count/validation walk.
    pub fn finish(self) -> Result<FileArtifact<'a>, RejectedFile<'a>> {
        match self.canonical.finish() {
            Ok(artifact) => Ok(FileArtifact::from_producer(artifact, self.facts)),
            Err(artifact) => Err(RejectedFile {
                artifact,
                facts: Box::new(self.facts),
            }),
        }
    }
}

pub(crate) struct DeclarationSite<'a> {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub name: &'a str,
    pub span: Span,
    pub namespace: Namespace,
    pub kind: DeclarationKind,
    pub initializer: InitializerKind,
    pub import_source: Option<String>,
    pub imported_name: Option<String>,
    pub direct_program: bool,
}

impl<'a> Facts<'a> {
    pub(crate) fn new(source: &str) -> Self {
        Self {
            units: Vec::new(),
            scopes: alloc::vec![Scope {
                id: ScopeId(0),
                parent: None,
                span: Span::new(0, source.len() as u32),
            }],
            declarations: Vec::new(),
            references: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            issues: Vec::new(),
            expressions: SideTable::new(),
            template_issues: Vec::new(),
            template_walk: super::template::TemplateWalk::Idle,
            setup_annotations: None,
            names: alloc::vec![Names::default()],
        }
    }

    pub(crate) fn issue(&mut self, unit: ScriptUnitId, span: Span, kind: FileIssueKind) {
        self.issues.push(FileIssue { unit, span, kind });
    }

    pub(crate) fn unit(
        &mut self,
        index: u32,
        span: Span,
        profile: ScriptProfile,
        nested: bool,
        origin: ProgramOrigin,
    ) -> Option<(ScriptUnitId, ScopeId)> {
        let unit = ScriptUnitId(index);
        if self.units.iter().any(|existing| existing.id == unit) {
            self.issue(unit, span, FileIssueKind::DuplicateUnit);
            return None;
        }
        let scope = if nested {
            self.child_scope(unit, ScopeId(0), span)?
        } else {
            ScopeId(0)
        };
        self.units
            .push(ScriptUnit::new(unit, span, profile, scope, origin));
        Some((unit, scope))
    }

    /// Only the checked language producer introduces a real lexical boundary.
    pub(crate) fn child_scope(
        &mut self,
        unit: ScriptUnitId,
        parent: ScopeId,
        span: Span,
    ) -> Option<ScopeId> {
        if self.scopes.get(parent.index() as usize).is_none() {
            self.issue(unit, span, FileIssueKind::InvalidSpan);
            return None;
        }
        let Ok(index) = u32::try_from(self.scopes.len()) else {
            self.issue(unit, span, FileIssueKind::BindingLimit);
            return None;
        };
        let id = ScopeId(index);
        self.scopes.push(Scope {
            id,
            parent: Some(parent),
            span,
        });
        self.names.push(Names::default());
        Some(id)
    }

    pub(crate) fn declare(&mut self, site: DeclarationSite<'_>) -> Option<BindingId> {
        let Some(names) = self.names.get_mut(site.scope.index() as usize) else {
            self.issue(site.unit, site.span, FileIssueKind::InvalidSpan);
            return None;
        };
        let map = match site.namespace {
            Namespace::Value => &mut names.value_names,
            Namespace::Type => &mut names.type_names,
        };
        if map.contains_key(site.name) {
            self.issue(site.unit, site.span, FileIssueKind::DuplicateDeclaration);
            return None;
        }
        let Ok(index) = u32::try_from(self.declarations.len()) else {
            self.issue(site.unit, site.span, FileIssueKind::BindingLimit);
            return None;
        };
        let id = BindingId::new(index);
        map.insert(String::from(site.name), id);
        self.declarations.push(Declaration {
            id,
            unit: site.unit,
            scope: site.scope,
            name: String::from(site.name),
            span: site.span,
            namespace: site.namespace,
            kind: site.kind,
            initializer: site.initializer,
            import_source: site.import_source,
            imported_name: site.imported_name,
            direct_program: site.direct_program,
        });
        if site.direct_program
            && site.namespace == Namespace::Value
            && super::vue::reserved(site.name)
            && let Some(unit) = self.units.iter_mut().find(|unit| unit.id == site.unit)
        {
            unit.origin.reserved_binding = true;
        }
        Some(id)
    }

    pub(crate) fn lookup(
        &self,
        scope: ScopeId,
        name: &str,
        namespace: Namespace,
    ) -> Option<BindingId> {
        lookup(&self.names, &self.scopes, scope, name, namespace)
    }

    /// Close only accepted pending rows; no AST/tree/sealing walk is added.
    pub(crate) fn resolve_references(&mut self, first: usize, first_export: usize) {
        for reference in self.references.iter_mut().skip(first) {
            if let Some(binding) = lookup(
                &self.names,
                &self.scopes,
                reference.scope,
                reference.name.as_str(),
                reference.namespace,
            ) {
                reference.target = ReferenceTarget::Resolved(binding);
            } else {
                self.issues.push(FileIssue {
                    unit: reference.unit,
                    span: reference.span,
                    kind: FileIssueKind::UnresolvedReference,
                });
            }
        }
        for export in self.exports.iter_mut().skip(first_export) {
            if let Some(reference) = export
                .local_reference
                .and_then(|index| self.references.get(index))
                && let ReferenceTarget::Resolved(binding) = reference.target
            {
                export.local = Some(binding);
            }
        }
    }
}

fn lookup(
    names: &[Names],
    scopes: &[Scope],
    mut scope: ScopeId,
    name: &str,
    namespace: Namespace,
) -> Option<BindingId> {
    loop {
        let names = names.get(scope.index() as usize)?;
        let map = match namespace {
            Namespace::Value => &names.value_names,
            Namespace::Type => &names.type_names,
        };
        if let Some(binding) = map.get(name) {
            return Some(*binding);
        }
        scope = scopes.get(scope.index() as usize)?.parent?;
    }
}

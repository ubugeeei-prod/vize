//! A position index for repeated queries against one immutable File.
//! Only row locations are cached. Handles always borrow the original File.

mod intervals;
use intervals::Intervals;

use super::{
    BindingRef, FileArtifact, Model, PositionQueryError, ReferenceRef, ScopeRef, ScriptUnit,
    contains, query_declaration, query_reference, scope_descends, unique,
};
use crate::query::template::index::TemplatePositions;
use crate::query::{TemplateQueryError, TemplateSiteRef};
use core::ops::Deref;

/// The index and its File are kept together. A snapshot may own both, or a
/// short-lived tool session may index a File owned by its caller.
pub struct PositionIndex<F> {
    owner: F,
    rows: Rows,
}

impl<'a> PositionIndex<FileArtifact<'a>> {
    /// Take ownership of a finished File. Moving the snapshot preserves every row identity.
    #[must_use]
    pub fn from_file(file: FileArtifact<'a>) -> Self {
        let rows = Rows::new(&file);
        Self { owner: file, rows }
    }

    #[must_use]
    pub fn model(&self) -> IndexedModel<'_, 'a> {
        self.rows.model(&self.owner)
    }

    /// Drop the index and recover the original File.
    #[must_use]
    pub fn into_file(self) -> FileArtifact<'a> {
        self.owner
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Build a reusable index at the caller's first demand for position queries.
    #[must_use]
    pub fn index_positions(self) -> PositionIndex<&'f FileArtifact<'a>> {
        PositionIndex {
            owner: self.file,
            rows: Rows::new(self.file),
        }
    }
}

impl<'a> PositionIndex<&FileArtifact<'a>> {
    #[must_use]
    pub fn model(&self) -> IndexedModel<'_, 'a> {
        self.rows.model(self.owner)
    }
}

#[derive(Default)]
struct Rows {
    complete: bool,
    indexed_script: bool,
    units: Intervals<usize>,
    declarations: Intervals<usize>,
    references: Intervals<usize>,
    scopes: Intervals<usize>,
    template: TemplatePositions,
}

impl Rows {
    fn new(file: &FileArtifact<'_>) -> Self {
        if !file.is_complete() {
            return Self::default();
        }
        let mut rows = Self {
            complete: true,
            template: TemplatePositions::new(file),
            ..Self::default()
        };
        // Short contiguous tables are cheaper to scan. The paired position
        // profile measures the crossover using the same source and questions.
        let sites = file
            .units()
            .len()
            .saturating_add(file.scopes().len())
            .saturating_add(file.facts.declarations.len())
            .saturating_add(file.references().len());
        if sites >= 96 {
            rows.index_script(file);
        }
        rows
    }

    fn index_script(&mut self, file: &FileArtifact<'_>) {
        self.units = Intervals::from_rows(file.units(), |row| row.span);
        self.declarations = Intervals::from_rows(&file.facts.declarations, |row| row.span);
        self.references = Intervals::from_rows(file.references(), |row| row.span);
        self.scopes = Intervals::from_rows(file.scopes(), |row| row.span);
        self.indexed_script = true;
    }

    fn model<'f, 'a>(&'f self, file: &'f FileArtifact<'a>) -> IndexedModel<'f, 'a> {
        IndexedModel {
            scan: Model::new(file),
            rows: self,
        }
    }
}

/// A borrowed semantic model backed by the index. Enumeration and scope
/// lookup use the original Model and its original handles.
#[derive(Clone, Copy)]
pub struct IndexedModel<'f, 'a> {
    scan: Model<'f, 'a>,
    rows: &'f Rows,
}

impl<'f, 'a> Deref for IndexedModel<'f, 'a> {
    type Target = Model<'f, 'a>;
    fn deref(&self) -> &Self::Target {
        &self.scan
    }
}

impl<'f, 'a> IndexedModel<'f, 'a> {
    fn check(self, offset: u32) -> Result<(), PositionQueryError> {
        let file = self.scan.file;
        if !self.rows.complete {
            return Err(PositionQueryError::IncompleteFile);
        }
        let source = file.artifact().source();
        if offset as usize > source.len() {
            return Err(PositionQueryError::OutOfBounds);
        }
        if !source.is_char_boundary(offset as usize) {
            return Err(PositionQueryError::NotCharBoundary);
        }
        Ok(())
    }

    fn unit(self, offset: u32) -> Result<Option<&'f ScriptUnit>, PositionQueryError> {
        self.check(offset)?;
        let units = self.scan.file.units();
        if let [unit] = units {
            return Ok(contains(unit.span, offset).then_some(unit));
        }
        if !self.rows.indexed_script {
            return unique(units.iter().filter(|unit| contains(unit.span, offset)));
        }
        self.rows
            .units
            .unique(offset, |_, index| self.scan.file.units().get(*index))
    }

    fn reference(
        self,
        unit: &ScriptUnit,
        offset: u32,
    ) -> Result<Option<ReferenceRef<'f, 'a>>, PositionQueryError> {
        let file = self.scan.file;
        if !self.rows.indexed_script {
            return query_reference(file, unit, offset);
        }
        self.rows.references.unique(offset, |_, index| {
            file.references()
                .get(*index)
                .filter(|row| row.unit == unit.id)
                .map(|reference| ReferenceRef { file, reference })
        })
    }

    fn declaration(
        self,
        unit: &ScriptUnit,
        offset: u32,
    ) -> Result<Option<BindingRef<'f, 'a>>, PositionQueryError> {
        let file = self.scan.file;
        if !self.rows.indexed_script {
            return query_declaration(file, unit, offset);
        }
        self.rows
            .declarations
            .unique(offset, |_, index| {
                file.facts
                    .declarations
                    .get(*index)
                    .filter(|row| row.unit == unit.id)
            })
            .map(|row| row.and_then(|row| file.binding(row.id)))
    }

    /// The unique original reference covering this byte.
    #[inline]
    pub fn reference_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<ReferenceRef<'f, 'a>>, PositionQueryError> {
        let Some(unit) = self.unit(offset)? else {
            return Ok(None);
        };
        self.reference(unit, offset)
    }

    /// The original declaration, or the resolved target of a use.
    #[inline]
    pub fn binding_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<BindingRef<'f, 'a>>, PositionQueryError> {
        let Some(unit) = self.unit(offset)? else {
            return Ok(None);
        };
        match (
            self.declaration(unit, offset)?,
            self.reference(unit, offset)?,
        ) {
            (Some(_), Some(_)) => Err(PositionQueryError::AmbiguousSite),
            (Some(binding), None) => Ok(Some(binding)),
            (None, reference) => Ok(reference.and_then(ReferenceRef::binding)),
        }
    }

    /// The original name-site scope or deepest comparable containing scope.
    #[inline]
    pub fn scope_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<ScopeRef<'f, 'a>>, PositionQueryError> {
        if !self.rows.indexed_script {
            return self.scan.scope_at_offset(offset);
        }
        let Some(unit) = self.unit(offset)? else {
            return Ok(None);
        };
        let file = self.scan.file;
        let site = match (
            self.declaration(unit, offset)?,
            self.reference(unit, offset)?,
        ) {
            (Some(_), Some(_)) => return Err(PositionQueryError::AmbiguousSite),
            (Some(binding), None) => binding.declaration().map(|row| row.scope),
            (None, reference) => reference.map(|row| row.reference.scope),
        };
        let Some(mut scope) = file
            .scopes()
            .get(site.unwrap_or(unit.scope).index() as usize)
        else {
            return Ok(None);
        };
        if site.is_none() {
            self.rows.scopes.visit(offset, &mut |_, index| {
                let Some(candidate) = file.scopes().get(*index) else {
                    return Err(PositionQueryError::IncompleteFile);
                };
                if scope_descends(file, candidate.id, unit.scope) {
                    if scope_descends(file, candidate.id, scope.id) {
                        scope = candidate;
                    } else if !scope_descends(file, scope.id, candidate.id) {
                        return Err(PositionQueryError::AmbiguousSite);
                    }
                }
                Ok(())
            })?;
        }
        Ok(Some(ScopeRef { file, scope }))
    }

    /// The unique checked handler or loop-alias site at this byte.
    pub fn template_symbol_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<TemplateSiteRef<'f, 'a>>, TemplateQueryError> {
        self.check(offset).map_err(TemplateQueryError::Position)?;
        self.rows.template.at(self.scan.file, offset)
    }
}

pub(in crate::query) use intervals::Intervals as SiteIntervals;

#[cfg(test)]
pub(in crate::query) mod tests;

//! Borrowed script-family queries over original file-absolute UTF-8 sites.
//!
//! These scans use the existing completed observations, without parsing,
//! indexing or allocation. Template occurrence tables remain a separate API.

use super::{
    BindingRef, FileArtifact, Namespace, Reference, ReferenceTarget, Scope, ScopeId, ScriptUnit,
};
use vize_l0::Span;

/// A position query never treats structural `finish()` as semantic completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionQueryError {
    IncompleteFile,
    OutOfBounds,
    NotCharBoundary,
    /// More than one original unit/site, or incomparable scopes, covers the byte.
    AmbiguousSite,
}

/// An original use row and its actual File owner stay borrowed together.
///
/// ```compile_fail
/// use vize_l2::file::{FileArtifact, ReferenceRef};
/// fn substitute<'f, 'a>(file: &'f FileArtifact<'a>, row: ReferenceRef<'f, 'a>) {
///     let _ = ReferenceRef { file, reference: row.reference() };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// fn discard(file: FileArtifact<'_>, offset: u32) {
///     let row = file.reference_at_offset(offset).unwrap().unwrap();
///     drop(file);
///     let _ = row.binding();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ReferenceRef<'file, 'arena> {
    file: &'file FileArtifact<'arena>,
    reference: &'file Reference,
}

impl<'file, 'arena> ReferenceRef<'file, 'arena> {
    #[must_use]
    pub fn file(self) -> &'file FileArtifact<'arena> {
        self.file
    }

    #[must_use]
    pub fn reference(self) -> &'file Reference {
        self.reference
    }

    /// Resolve the original target only inside this owner; no name fallback.
    #[must_use]
    pub fn binding(self) -> Option<BindingRef<'file, 'arena>> {
        match self.reference.target {
            ReferenceTarget::Resolved(id) => self.file.binding(id),
            ReferenceTarget::Unresolved => None,
        }
    }
}

/// A recorded scope with lookup constrained to the same completed File.
///
/// ```compile_fail
/// use vize_l2::file::{FileArtifact, ScopeRef};
/// fn substitute<'f, 'a>(file: &'f FileArtifact<'a>, row: ScopeRef<'f, 'a>) {
///     let _ = ScopeRef { file, scope: row.scope() };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ScopeRef<'file, 'arena> {
    file: &'file FileArtifact<'arena>,
    scope: &'file Scope,
}

impl<'file, 'arena> ScopeRef<'file, 'arena> {
    #[must_use]
    pub fn file(self) -> &'file FileArtifact<'arena> {
        self.file
    }

    #[must_use]
    pub fn scope(self) -> &'file Scope {
        self.scope
    }

    #[must_use]
    pub fn lookup(self, name: &str, namespace: Namespace) -> Option<BindingRef<'file, 'arena>> {
        self.file.lookup(self.scope.id, name, namespace)
    }
}

impl<'arena> FileArtifact<'arena> {
    /// Find the unique original script use covering a file-absolute byte.
    ///
    /// All queries require actual `is_complete()` and a UTF-8 boundary first.
    /// Spans are half-open: EOF and zero-width sites never match. Source outside
    /// original Program units, template uses, imported names and export aliases
    /// are not synthesized into references. Multiple unit/site observations
    /// return `AmbiguousSite` independently of their recording order.
    pub fn reference_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<ReferenceRef<'_, 'arena>>, PositionQueryError> {
        let Some(unit) = self.query_unit(offset)? else {
            return Ok(None);
        };
        self.query_reference(unit, offset)
    }

    /// Find an original declaration, or the actual target of an original use.
    ///
    /// The returned binding retains this File. Type/value namespace, import
    /// local aliases, shorthand and export-local uses come from their genuine
    /// rows; an exported public alias or remote import spelling has no invented
    /// local target. Overlapping distinct sites are refused, never prioritized.
    pub fn binding_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<BindingRef<'_, 'arena>>, PositionQueryError> {
        let Some(unit) = self.query_unit(offset)? else {
            return Ok(None);
        };
        let declaration = self.query_declaration(unit, offset)?;
        let reference = self.query_reference(unit, offset)?;
        match (declaration, reference) {
            (Some(_), Some(_)) => Err(PositionQueryError::AmbiguousSite),
            (Some(binding), None) => Ok(Some(binding)),
            (None, Some(reference)) => Ok(reference.binding()),
            (None, None) => Ok(None),
        }
    }

    /// Return the original name-site scope, otherwise the deepest recorded
    /// containing scope in the unique Program unit.
    ///
    /// A function's declared name retains its parent scope even when the
    /// child's recorded full-function span covers that name. Other positions
    /// use actual recorded span containment and ancestry; they do not infer
    /// visibility extents from introductions or cross opaque source gaps.
    /// Incomparable overlapping scopes are ambiguous. This script-family query
    /// does not invent a scope for template expressions or unparsed SFC blocks.
    pub fn scope_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<ScopeRef<'_, 'arena>>, PositionQueryError> {
        let Some(unit) = self.query_unit(offset)? else {
            return Ok(None);
        };
        let declaration = self.query_declaration(unit, offset)?;
        let reference = self.query_reference(unit, offset)?;
        let site_scope = match (declaration, reference) {
            (Some(_), Some(_)) => return Err(PositionQueryError::AmbiguousSite),
            (Some(binding), None) => binding.declaration().map(|row| row.scope),
            (None, Some(reference)) => Some(reference.reference.scope),
            (None, None) => None,
        };
        let Some(mut scope) = self
            .facts
            .scopes
            .get(site_scope.unwrap_or(unit.scope).index() as usize)
        else {
            return Ok(None);
        };
        if site_scope.is_none() {
            for candidate in &self.facts.scopes {
                if !contains(candidate.span, offset)
                    || !self.scope_descends(candidate.id, unit.scope)
                {
                    continue;
                }
                if self.scope_descends(candidate.id, scope.id) {
                    scope = candidate;
                } else if !self.scope_descends(scope.id, candidate.id) {
                    return Err(PositionQueryError::AmbiguousSite);
                }
            }
        }
        Ok(Some(ScopeRef { file: self, scope }))
    }

    fn query_unit(&self, offset: u32) -> Result<Option<&ScriptUnit>, PositionQueryError> {
        if !self.is_complete() {
            return Err(PositionQueryError::IncompleteFile);
        }
        let source = self.artifact().source();
        if offset as usize > source.len() {
            return Err(PositionQueryError::OutOfBounds);
        }
        if !source.is_char_boundary(offset as usize) {
            return Err(PositionQueryError::NotCharBoundary);
        }
        unique(
            self.units()
                .iter()
                .filter(|unit| contains(unit.span, offset)),
        )
    }

    fn query_reference(
        &self,
        unit: &ScriptUnit,
        offset: u32,
    ) -> Result<Option<ReferenceRef<'_, 'arena>>, PositionQueryError> {
        unique(
            self.references()
                .iter()
                .filter(|row| row.unit == unit.id && contains(row.span, offset)),
        )
        .map(|row| {
            row.map(|reference| ReferenceRef {
                file: self,
                reference,
            })
        })
    }

    fn query_declaration(
        &self,
        unit: &ScriptUnit,
        offset: u32,
    ) -> Result<Option<BindingRef<'_, 'arena>>, PositionQueryError> {
        unique(
            self.facts
                .declarations
                .iter()
                .filter(|row| row.unit == unit.id && contains(row.span, offset)),
        )
        .map(|row| row.and_then(|declaration| self.binding(declaration.id)))
    }

    fn scope_descends(&self, mut scope: ScopeId, ancestor: ScopeId) -> bool {
        // Parents are recorded before their children by the existing factory.
        // A bounded read also prevents malformed ancestry from looping.
        for _ in 0..self.facts.scopes.len() {
            if scope == ancestor {
                return true;
            }
            let Some(parent) = self
                .facts
                .scopes
                .get(scope.index() as usize)
                .and_then(|row| row.parent)
            else {
                return false;
            };
            scope = parent;
        }
        false
    }
}

fn contains(span: Span, offset: u32) -> bool {
    span.start <= offset && offset < span.end
}

fn unique<T>(mut rows: impl Iterator<Item = T>) -> Result<Option<T>, PositionQueryError> {
    let row = rows.next();
    if rows.next().is_some() {
        Err(PositionQueryError::AmbiguousSite)
    } else {
        Ok(row)
    }
}

#[cfg(test)]
mod tests;

//! Borrowed script-family queries over original file-absolute UTF-8 sites.
//!
//! These scans use the existing completed observations, without parsing,
//! indexing or allocation. Template occurrence tables are a separate family.

use super::Model;
use crate::file::{
    BindingRef, L2Ir, Namespace, Reference, ReferenceTarget, Scope, ScopeId, ScriptUnit,
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
/// use vize_l2::{file::L2Ir, query::ReferenceRef};
/// fn substitute<'f, 'a>(l2ir: &'f L2Ir<'a>, row: ReferenceRef<'f, 'a>) {
///     let _ = ReferenceRef { l2ir, reference: row.reference() };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::{file::L2Ir, query::Model};
/// fn discard(l2ir: L2Ir<'_>, offset: u32) {
///     let row = Model::new(&l2ir).reference_at_offset(offset).unwrap().unwrap();
///     drop(l2ir);
///     let _ = row.binding();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ReferenceRef<'file, 'arena> {
    l2ir: &'file L2Ir<'arena>,
    reference: &'file Reference,
}

impl<'file, 'arena> ReferenceRef<'file, 'arena> {
    #[must_use]
    pub fn l2ir(self) -> &'file L2Ir<'arena> {
        self.l2ir
    }

    #[must_use]
    pub fn reference(self) -> &'file Reference {
        self.reference
    }

    /// Resolve the original target only inside this owner; no name fallback.
    #[must_use]
    pub fn binding(self) -> Option<BindingRef<'file, 'arena>> {
        match self.reference.target {
            ReferenceTarget::Resolved(id) => self.l2ir.binding(id),
            ReferenceTarget::Unresolved => None,
        }
    }
}

/// A recorded scope with lookup constrained to the same completed File.
///
/// ```compile_fail
/// use vize_l2::{file::L2Ir, query::ScopeRef};
/// fn substitute<'f, 'a>(l2ir: &'f L2Ir<'a>, row: ScopeRef<'f, 'a>) {
///     let _ = ScopeRef { l2ir, scope: row.scope() };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ScopeRef<'file, 'arena> {
    l2ir: &'file L2Ir<'arena>,
    scope: &'file Scope,
}

impl<'file, 'arena> ScopeRef<'file, 'arena> {
    #[must_use]
    pub fn l2ir(self) -> &'file L2Ir<'arena> {
        self.l2ir
    }

    #[must_use]
    pub fn scope(self) -> &'file Scope {
        self.scope
    }

    /// The recorded parent scope in the same File; the File root has none.
    #[must_use]
    pub fn parent(self) -> Option<Self> {
        let scope = self
            .l2ir
            .facts
            .scopes
            .get(self.scope.parent?.index() as usize)?;
        Some(Self {
            l2ir: self.l2ir,
            scope,
        })
    }

    #[must_use]
    pub fn lookup(self, name: &str, namespace: Namespace) -> Option<BindingRef<'file, 'arena>> {
        self.l2ir.lookup(self.scope.id, name, namespace)
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Find the unique original script use covering a file-absolute byte.
    ///
    /// All queries require actual `is_complete()` and a UTF-8 boundary first.
    /// Spans are half-open: EOF and zero-width sites never match. Source outside
    /// original Program units, template uses, imported names and export aliases
    /// are not synthesized into references. Multiple unit/site observations
    /// return `AmbiguousSite` independently of their recording order.
    pub fn reference_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<ReferenceRef<'f, 'a>>, PositionQueryError> {
        let Some(unit) = query_unit(self.l2ir, offset)? else {
            return Ok(None);
        };
        query_reference(self.l2ir, unit, offset)
    }

    /// Find an original declaration, or the actual target of an original use.
    ///
    /// The returned binding retains this File. Type/value namespace, import
    /// local aliases, shorthand and export-local uses come from their genuine
    /// rows; an exported public alias or remote import spelling has no invented
    /// local target. Overlapping distinct sites are refused, never prioritized.
    pub fn binding_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<BindingRef<'f, 'a>>, PositionQueryError> {
        let Some(unit) = query_unit(self.l2ir, offset)? else {
            return Ok(None);
        };
        let declaration = query_declaration(self.l2ir, unit, offset)?;
        let reference = query_reference(self.l2ir, unit, offset)?;
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
        self,
        offset: u32,
    ) -> Result<Option<ScopeRef<'f, 'a>>, PositionQueryError> {
        let file = self.l2ir;
        let Some(unit) = query_unit(file, offset)? else {
            return Ok(None);
        };
        let declaration = query_declaration(file, unit, offset)?;
        let reference = query_reference(file, unit, offset)?;
        let site_scope = match (declaration, reference) {
            (Some(_), Some(_)) => return Err(PositionQueryError::AmbiguousSite),
            (Some(binding), None) => binding.declaration().map(|row| row.scope),
            (None, Some(reference)) => Some(reference.reference.scope),
            (None, None) => None,
        };
        let Some(mut scope) = file
            .facts
            .scopes
            .get(site_scope.unwrap_or(unit.scope).index() as usize)
        else {
            return Ok(None);
        };
        if site_scope.is_none() {
            for candidate in &file.facts.scopes {
                if !contains(candidate.span, offset)
                    || !scope_descends(file, candidate.id, unit.scope)
                {
                    continue;
                }
                if scope_descends(file, candidate.id, scope.id) {
                    scope = candidate;
                } else if !scope_descends(file, scope.id, candidate.id) {
                    return Err(PositionQueryError::AmbiguousSite);
                }
            }
        }
        Ok(Some(ScopeRef { l2ir: file, scope }))
    }

    /// Visit the original script uses whose actual target is `binding`.
    ///
    /// Table order; no allocation. A binding of another File is never
    /// reinterpreted by its numeric id: it matches nothing.
    pub fn for_each_reference_to(
        self,
        binding: BindingRef<'_, '_>,
        mut visit: impl FnMut(ReferenceRef<'f, 'a>),
    ) {
        if !core::ptr::eq(self.l2ir, binding.l2ir()) {
            return;
        }
        let target = ReferenceTarget::Resolved(binding.id());
        for reference in self.l2ir.references() {
            if reference.target == target {
                visit(ReferenceRef {
                    l2ir: self.l2ir,
                    reference,
                });
            }
        }
    }
}

fn query_unit<'f>(
    l2ir: &'f L2Ir<'_>,
    offset: u32,
) -> Result<Option<&'f ScriptUnit>, PositionQueryError> {
    if !l2ir.is_complete() {
        return Err(PositionQueryError::IncompleteFile);
    }
    let source = l2ir.artifact().source();
    if offset as usize > source.len() {
        return Err(PositionQueryError::OutOfBounds);
    }
    if !source.is_char_boundary(offset as usize) {
        return Err(PositionQueryError::NotCharBoundary);
    }
    unique(
        l2ir.units()
            .iter()
            .filter(|unit| contains(unit.span, offset)),
    )
}

fn query_reference<'f, 'a>(
    l2ir: &'f L2Ir<'a>,
    unit: &ScriptUnit,
    offset: u32,
) -> Result<Option<ReferenceRef<'f, 'a>>, PositionQueryError> {
    unique(
        l2ir.references()
            .iter()
            .filter(|row| row.unit == unit.id && contains(row.span, offset)),
    )
    .map(|row| row.map(|reference| ReferenceRef { l2ir, reference }))
}

fn query_declaration<'f, 'a>(
    l2ir: &'f L2Ir<'a>,
    unit: &ScriptUnit,
    offset: u32,
) -> Result<Option<BindingRef<'f, 'a>>, PositionQueryError> {
    unique(
        l2ir.facts
            .declarations
            .iter()
            .filter(|row| row.unit == unit.id && contains(row.span, offset)),
    )
    .map(|row| row.and_then(|declaration| l2ir.binding(declaration.id)))
}

fn scope_descends(l2ir: &L2Ir<'_>, mut scope: ScopeId, ancestor: ScopeId) -> bool {
    // Parents are recorded before their children by the existing factory.
    // A bounded read also prevents malformed ancestry from looping.
    for _ in 0..l2ir.facts.scopes.len() {
        if scope == ancestor {
            return true;
        }
        let Some(parent) = l2ir
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

#[cfg(any(test, feature = "position-index"))]
pub(in crate::query) mod index;

#[cfg(feature = "template-operand-queries")]
impl<'f, 'a> ScopeRef<'f, 'a> {
    pub(super) fn from_id(l2ir: &'f L2Ir<'a>, id: ScopeId) -> Option<Self> {
        let scope = l2ir.scopes().get(id.index() as usize)?;
        (scope.id == id).then_some(Self { l2ir, scope })
    }
}

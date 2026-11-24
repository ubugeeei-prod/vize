//! Script semantic products selected at one byte in the same L2IR.
use super::{
    BindingRef, IndexedModel, PositionQueryError, ReferenceRef, ScopeRef, ScriptUnit, contains,
    scope_descends,
};
use crate::file::{Scope, ScopeId};

/// The three independent script answers at one source byte. Each successful
/// handle borrows its original row and file for the lifetime of the index.
#[derive(Clone, Copy)]
#[must_use]
pub struct ScriptPosition<'file, 'arena> {
    pub reference: Result<Option<ReferenceRef<'file, 'arena>>, PositionQueryError>,
    pub binding: Result<Option<BindingRef<'file, 'arena>>, PositionQueryError>,
    pub scope: Result<Option<ScopeRef<'file, 'arena>>, PositionQueryError>,
}

#[derive(Clone, Copy)]
pub(super) enum NameSite<D, R> {
    Vacant,
    Declaration(D),
    Reference(R),
}

impl<D, R> NameSite<D, R> {
    #[inline]
    fn select(declaration: Option<D>, reference: Option<R>) -> Result<Self, PositionQueryError> {
        match (declaration, reference) {
            (Some(_), Some(_)) => Err(PositionQueryError::AmbiguousSite),
            (Some(row), None) => Ok(Self::Declaration(row)),
            (None, Some(row)) => Ok(Self::Reference(row)),
            (None, None) => Ok(Self::Vacant),
        }
    }
}

impl<'f, 'a> NameSite<BindingRef<'f, 'a>, ReferenceRef<'f, 'a>> {
    #[inline]
    pub(super) fn binding(self) -> Option<BindingRef<'f, 'a>> {
        match self {
            Self::Declaration(binding) => Some(binding),
            Self::Reference(reference) => reference.binding(),
            Self::Vacant => None,
        }
    }

    #[inline]
    pub(super) fn scope(self) -> Option<ScopeId> {
        match self {
            Self::Declaration(binding) => binding.declaration().map(|row| row.scope),
            Self::Reference(reference) => Some(reference.reference.scope),
            Self::Vacant => None,
        }
    }
}

impl<'f, 'a> IndexedModel<'f, 'a> {
    /// Select a use, its declaration or resolved binding, and the lexical scope
    /// together. An ambiguous declaration or scope leaves an independent
    /// reference answer available to the caller.
    pub fn script_position_at_offset(self, offset: u32) -> ScriptPosition<'f, 'a> {
        let unit = self.unit(offset);
        let Ok(Some(unit)) = unit else {
            return ScriptPosition {
                reference: unit.map(|_| None),
                binding: unit.map(|_| None),
                scope: unit.map(|_| None),
            };
        };
        let reference = self.reference(unit, offset);
        let site = self
            .declaration(unit, offset)
            .and_then(|declaration| NameSite::select(declaration, reference?));
        ScriptPosition {
            reference,
            binding: site.map(NameSite::binding),
            scope: site.and_then(|site| self.scope_from_site(unit, offset, site.scope())),
        }
    }

    #[inline]
    pub(super) fn name_site(
        self,
        unit: &ScriptUnit,
        offset: u32,
    ) -> Result<NameSite<BindingRef<'f, 'a>, ReferenceRef<'f, 'a>>, PositionQueryError> {
        NameSite::select(
            self.declaration(unit, offset)?,
            self.reference(unit, offset)?,
        )
    }

    #[inline]
    pub(super) fn scope_from_site(
        self,
        unit: &ScriptUnit,
        offset: u32,
        site: Option<ScopeId>,
    ) -> Result<Option<ScopeRef<'f, 'a>>, PositionQueryError> {
        let file = self.scan.file;
        let Some(mut scope) = file
            .scopes()
            .get(site.unwrap_or(unit.scope).index() as usize)
        else {
            return Ok(None);
        };
        if site.is_none() {
            let mut consider = |candidate: &'f Scope| {
                if scope_descends(file, candidate.id, unit.scope) {
                    if scope_descends(file, candidate.id, scope.id) {
                        scope = candidate;
                    } else if !scope_descends(file, scope.id, candidate.id) {
                        return Err(PositionQueryError::AmbiguousSite);
                    }
                }
                Ok(())
            };
            if self.rows.indexed_script {
                self.rows.scopes.visit(offset, &mut |_, index| {
                    consider(
                        file.scopes()
                            .get(*index)
                            .ok_or(PositionQueryError::IncompleteFile)?,
                    )
                })?;
            } else {
                for candidate in file
                    .scopes()
                    .iter()
                    .filter(|row| contains(row.span, offset))
                {
                    consider(candidate)?;
                }
            }
        }
        Ok(Some(ScopeRef { file, scope }))
    }
}

#[cfg(test)]
mod tests;

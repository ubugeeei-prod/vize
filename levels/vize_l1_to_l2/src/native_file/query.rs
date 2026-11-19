//! Original SFC queries borrow the admitted assembly, not diagnostic coordinates.

use super::NativeSfc;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    file::{
        BindingRef, FileArtifact, FileResolution, PositionQueryError, Reference, ReferenceTarget,
        ScopeId,
    },
    resolution::{Occurrence, Usage},
};

/// An original-owner query keeps source and semantic refusals explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePositionQueryError {
    File(PositionQueryError),
    OriginalMembership,
    Projection,
    ForeignBinding,
}

#[derive(Clone, Copy)]
enum Site<'o, 'a> {
    Script(&'o Reference),
    Template(FileResolution<'o, 'a>, &'o Occurrence<'a>),
}

/// A genuine original use, its authored spelling and actual File stay borrowed.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSfcObservation;
/// fn discard(owner: NativeSfcObservation<'_>, offset: u32) {
///     let native = owner.admitted().unwrap();
///     let use_site = native.reference_at_offset(offset).unwrap().unwrap();
///     drop(owner);
///     let _ = use_site.binding();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct NativeReferenceRef<'o, 'a> {
    binding: BindingRef<'o, 'a>,
    scope: ScopeId,
    span: Span,
    site: Site<'o, 'a>,
}

impl<'o, 'a> NativeReferenceRef<'o, 'a> {
    #[must_use]
    pub fn file(self) -> &'o FileArtifact<'a> {
        self.binding.file()
    }
    #[must_use]
    pub fn binding(self) -> BindingRef<'o, 'a> {
        self.binding
    }
    #[must_use]
    pub fn span(self) -> Span {
        self.span
    }
    /// The scope captured by the original Program/expression factory.
    #[must_use]
    pub fn scope(self) -> ScopeId {
        self.scope
    }
    #[must_use]
    pub fn usage(self) -> Usage {
        match self.site {
            Site::Script(row) => row.usage,
            Site::Template(_, row) => row.usage,
        }
    }
    #[must_use]
    pub fn script(self) -> Option<&'o Reference> {
        match self.site {
            Site::Script(row) => Some(row),
            Site::Template(_, _) => None,
        }
    }
    #[must_use]
    pub fn template(self) -> Option<&'o Occurrence<'a>> {
        match self.site {
            Site::Script(_) => None,
            Site::Template(_, row) => Some(row),
        }
    }
    #[must_use]
    pub fn node(self) -> Option<NodeId> {
        match self.site {
            Site::Script(_) => None,
            Site::Template(resolution, _) => Some(resolution.node()),
        }
    }
}

impl<'o, 'a> NativeSfc<'o, 'a> {
    /// Find one original script/template use at a file-absolute UTF-8 boundary.
    /// Names are half-open; EOF, comments and opaque blocks have no invented use.
    /// Any overlapping observations refuse, including identical binding IDs.
    pub fn reference_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<NativeReferenceRef<'o, 'a>>, NativePositionQueryError> {
        self.file()
            .file()
            .reference_at_offset(offset)
            .map_err(NativePositionQueryError::File)?;
        let mut found = None;
        let mut ambiguous = false;
        self.for_each_reference(|row| {
            if row.span.start <= offset && offset < row.span.end {
                ambiguous |= found.is_some();
                found = Some(row);
            }
        })?;
        if ambiguous {
            return Err(NativePositionQueryError::File(
                PositionQueryError::AmbiguousSite,
            ));
        }
        Ok(found)
    }

    /// Resolve an original declaration/use inside this same completed File.
    /// Remote import spellings and public export aliases have no local fallback.
    pub fn binding_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<BindingRef<'o, 'a>>, NativePositionQueryError> {
        let script = self
            .file()
            .file()
            .binding_at_offset(offset)
            .map_err(NativePositionQueryError::File)?;
        let Some(reference) = self.reference_at_offset(offset)? else {
            return Ok(script);
        };
        if reference.template().is_some() && script.is_some() {
            return Err(NativePositionQueryError::File(
                PositionQueryError::AmbiguousSite,
            ));
        }
        Ok(Some(reference.binding()))
    }

    /// Visit genuine uses of this owner's binding without allocation or sorting.
    /// Table order is unspecified. On refusal the caller must discard any work
    /// already visited; a partial response must never be published as success.
    pub fn for_each_reference_to(
        &self,
        binding: BindingRef<'_, '_>,
        mut visit: impl FnMut(NativeReferenceRef<'o, 'a>),
    ) -> Result<(), NativePositionQueryError> {
        if !core::ptr::eq(binding.file(), self.file().file()) {
            return Err(NativePositionQueryError::ForeignBinding);
        }
        self.for_each_reference(|row| {
            if row.binding.id() == binding.id() {
                visit(row);
            }
        })
    }

    fn for_each_reference(
        &self,
        mut visit: impl FnMut(NativeReferenceRef<'o, 'a>),
    ) -> Result<(), NativePositionQueryError> {
        let file = self.file().file();
        if !file.is_complete() {
            return Err(NativePositionQueryError::File(
                PositionQueryError::IncompleteFile,
            ));
        }
        for row in file.references() {
            let ReferenceTarget::Resolved(id) = row.target else {
                return Err(NativePositionQueryError::Projection);
            };
            let binding = file
                .binding(id)
                .ok_or(NativePositionQueryError::Projection)?;
            visit(NativeReferenceRef {
                binding,
                scope: row.scope,
                span: row.span,
                site: Site::Script(row),
            });
        }
        let Some(template) = self.observation().template() else {
            return Ok(());
        };
        for embed in template.embeds() {
            let resolution = file
                .expression(
                    embed
                        .node
                        .ok_or(NativePositionQueryError::OriginalMembership)?,
                )
                .ok_or(NativePositionQueryError::OriginalMembership)?;
            let table = resolution
                .table()
                .ok_or(NativePositionQueryError::OriginalMembership)?;
            let expression = table.expression();
            if !core::ptr::eq(
                expression.ast,
                embed
                    .syntax
                    .expression()
                    .ok_or(NativePositionQueryError::OriginalMembership)?,
            ) || !core::ptr::eq(expression.source, embed.syntax.source().text())
                || !core::ptr::eq(
                    embed.syntax.source().authored_root(),
                    file.artifact().source(),
                )
                || expression.span != embed.syntax.source().span()
            {
                return Err(NativePositionQueryError::OriginalMembership);
            }
            let scope = resolution
                .scope()
                .ok_or(NativePositionQueryError::OriginalMembership)?;
            for row in table.occurrences() {
                let span = expression
                    .authored_span(row.span)
                    .ok_or(NativePositionQueryError::Projection)?;
                let binding = resolution
                    .binding(row.binding)
                    .ok_or(NativePositionQueryError::Projection)?;
                if !resolution.accepts(binding) {
                    return Err(NativePositionQueryError::OriginalMembership);
                }
                visit(NativeReferenceRef {
                    binding,
                    scope,
                    span,
                    site: Site::Template(resolution, row),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

//! Reuse the same interval search as script and handler positions. Keep only
//! compact row indices, so a snapshot can move together with its File.
use super::{
    ExpressionReferenceRef, FileArtifact, NodeId, TemplateQueryError, check_template_offset,
    visit_rows,
};
use crate::query::position::index::SiteIntervals;
use core::{
    borrow::Borrow,
    cell::{Cell, OnceCell},
};

type Prepared = Result<Option<SiteIntervals<(NodeId, u32)>>, TemplateQueryError>;

/// A demand-built index for one immutable File. The first position request
/// scans the original rows. Repeated requests on larger snapshots share an
/// interval index; short scans need no heap allocation.
pub struct ExpressionPositionIndex<F> {
    owner: F,
    visited: Cell<bool>,
    rows: OnceCell<Prepared>,
}

impl<'f, 'a> ExpressionPositionIndex<&'f FileArtifact<'a>> {
    /// Borrow one exact File, without inspecting its rows.
    #[must_use]
    pub fn new(file: &'f FileArtifact<'a>) -> Self {
        Self::attach(file)
    }
}

impl<'a> ExpressionPositionIndex<FileArtifact<'a>> {
    /// Own the File and its index in one movable snapshot.
    #[must_use]
    pub fn from_file(file: FileArtifact<'a>) -> Self {
        Self::attach(file)
    }
}

impl<F> ExpressionPositionIndex<F> {
    fn attach(owner: F) -> Self {
        Self {
            owner,
            visited: Cell::new(false),
            rows: OnceCell::new(),
        }
    }

    /// Return the original owner, dropping only the index.
    pub fn into_file(self) -> F {
        self.owner
    }

    /// Reborrow the exact occurrence and binding in the indexed File.
    pub fn expression_reference_at_offset<'f, 'a>(
        &'f self,
        offset: u32,
    ) -> Result<Option<ExpressionReferenceRef<'f, 'a>>, TemplateQueryError>
    where
        F: Borrow<FileArtifact<'a>>,
    {
        let file = self.owner.borrow();
        if !self.visited.replace(true) {
            return super::Model::new(file).expression_reference_at_offset(offset);
        }
        check_template_offset(file, offset)?;
        let prepared = self.rows.get_or_init(|| prepare(file));
        let Some(rows) = prepared.as_ref().map_err(|error| *error)? else {
            return super::scan_at(file, offset);
        };
        let Some((span, &(node, index))) = rows
            .unique(offset, |span, key| Some((span, key)))
            .map_err(TemplateQueryError::Position)?
        else {
            return Ok(None);
        };
        let resolved = file
            .facts
            .expressions
            .get(node)
            .ok_or(TemplateQueryError::OriginalMembership)?;
        let occurrence = resolved
            .table
            .occurrences()
            .get(index as usize)
            .ok_or(TemplateQueryError::OriginalMembership)?;
        let binding = file
            .binding(occurrence.binding)
            .ok_or(TemplateQueryError::OriginalMembership)?;
        Ok(Some(ExpressionReferenceRef {
            binding,
            scope: resolved.scope,
            node,
            span,
            occurrence,
        }))
    }
}

fn prepare(file: &FileArtifact<'_>) -> Prepared {
    // Count short tables as well: a single expression can contain many uses.
    const MIN_SITES: usize = 12;
    let count = file
        .facts
        .expressions
        .iter()
        .map(|(_, row)| row.table.occurrences().len())
        .sum();
    if count < MIN_SITES {
        return Ok(None);
    }
    SiteIntervals::try_from_sized_sites(count, |push| {
        visit_rows(file, |index, site| {
            let index = u32::try_from(index).map_err(|_| TemplateQueryError::Projection)?;
            push(site.span, (site.node, index));
            Ok(())
        })
    })
    .map(Some)
}

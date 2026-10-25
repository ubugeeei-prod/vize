//! Borrow original intrinsic leaves while the same Program and File stay alive.

use super::{NavigationRefusal, RetainedNavigation, Span};
use crate::source_project::navigation::{IntrinsicTagTarget, coordinates};
use tower_lsp::lsp_types::Position;
use vize_l2::{
    file::ScriptUnit,
    lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent, SyntaxEvent},
    resolution::{ResolutionErrorKind, SyntaxEdge, SyntaxKind},
};

#[derive(Default)]
pub(super) struct Tags<'a>(pub(super) Vec<(&'a str, Span)>);

impl<'a> FileObserver<'a> for Tags<'a> {
    type Checkpoint = usize;
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) -> usize {
        self.0.len()
    }
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn syntax(&mut self, event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if let SyntaxKind::Intrinsic(name) = event.kind
            && event.edge == SyntaxEdge::Leaf
        {
            self.0.push((name, event.span));
        }
        Ok(())
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.0.truncate(checkpoint);
    }
}

impl RetainedNavigation<'_, '_> {
    pub(super) fn intrinsic_tag(
        &self,
        position: Position,
    ) -> Result<Option<IntrinsicTagTarget>, NavigationRefusal> {
        let at = self.offset(position)?;
        let candidate = self
            .intrinsic_tags
            .partition_point(|(_, span)| span.start <= at)
            .checked_sub(1)
            .and_then(|index| self.intrinsic_tags.get(index));
        let Some((name, span)) = candidate.filter(|(_, span)| at < span.end) else {
            return Ok(None);
        };
        let source = self.snapshot.source();
        if source.get(span.start as usize..span.end as usize) != Some(*name) {
            return Err(NavigationRefusal::Projection);
        }
        Ok(Some(IntrinsicTagTarget {
            name: (*name).into(),
            range: coordinates::range(source, self.lines, *span)?,
        }))
    }
}

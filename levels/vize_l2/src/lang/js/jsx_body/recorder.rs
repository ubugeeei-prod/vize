use super::records::Record;
use crate::file::ScriptUnit;
use crate::lang::js::file::observer::{
    CallEvent, DeclaredEvent, FileObserver, StatementEvent, SyntaxEvent,
};
use crate::resolution::{ResolutionErrorKind, SyntaxEdge, SyntaxKind};
use alloc::vec::Vec;

pub(super) struct Recorder<'a> {
    pub records: Vec<Record<'a>>,
    frames: Vec<usize>,
    jsx_count: usize,
}

impl<'a> Recorder<'a> {
    pub(super) fn new() -> Self {
        Self {
            records: Vec::new(),
            frames: Vec::new(),
            jsx_count: 0,
        }
    }
    pub(super) fn frames_complete(&self) -> bool {
        self.frames.is_empty()
    }
    pub(super) fn jsx_count(&self) -> usize {
        self.jsx_count
    }
}

impl<'a> FileObserver<'a> for Recorder<'a> {
    type Checkpoint = (usize, usize, usize);
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) -> Self::Checkpoint {
        (self.records.len(), self.frames.len(), self.jsx_count)
    }
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn syntax(&mut self, event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if event.edge == SyntaxEdge::Leave {
            let index = self
                .frames
                .last()
                .copied()
                .ok_or(ResolutionErrorKind::InvalidSpan)?;
            let end = self.records.len();
            let record = self
                .records
                .get_mut(index)
                .ok_or(ResolutionErrorKind::InvalidSpan)?;
            if record.span != event.span
                || record.kind != event.kind
                || record.unit != event.unit
                || record.scope != event.scope
                || record.references.start > event.reference_boundary
            {
                return Err(ResolutionErrorKind::InvalidSpan);
            }
            record.references.end = event.reference_boundary;
            record.subtree_end = end;
            self.frames.pop();
            return Ok(());
        }
        if self.records.len() >= 4096 {
            return Err(ResolutionErrorKind::TraversalLimit);
        }
        let parent = self.frames.last().copied();
        if let Some(parent) = parent {
            let parent = self
                .records
                .get(parent)
                .ok_or(ResolutionErrorKind::InvalidSpan)?;
            if event.span.start < parent.span.start
                || event.span.end > parent.span.end
                || event.unit != parent.unit
                || event.scope != parent.scope
            {
                return Err(ResolutionErrorKind::InvalidSpan);
            }
        } else if !event.kind.is_expression() {
            return Err(ResolutionErrorKind::InvalidSpan);
        }
        let index = self.records.len();
        self.records.push(Record {
            kind: event.kind,
            span: event.span,
            unit: event.unit,
            scope: event.scope,
            parent,
            subtree_end: index + 1,
            references: event.reference_boundary..event.reference_boundary,
        });
        if matches!(event.kind, SyntaxKind::Element | SyntaxKind::Fragment) {
            self.jsx_count += 1;
        }
        if event.edge == SyntaxEdge::Enter {
            self.frames.push(index);
        }
        Ok(())
    }
    fn rollback(&mut self, (records, frames, count): Self::Checkpoint) {
        self.records.truncate(records);
        self.frames.truncate(frames);
        self.jsx_count = count;
    }
}
